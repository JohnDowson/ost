#![allow(clippy::empty_loop)]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(naked_functions)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(clippy::new_without_default)]

#[macro_use]
extern crate lazy_static;
extern crate alloc;

pub mod allocator;
pub mod clock;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod scheduler;
pub mod serial;
pub mod task;
pub mod thread;
pub mod userspace;
pub mod vga;

use core::panic::PanicInfo;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        let mut pics = interrupts::PICS.lock();
        pics.initialize();
        pics.write_masks(0, 0);
    };
    clock::init();
    x86_64::instructions::interrupts::enable();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
#[cfg(test)]
use bootloader::{entry_point, BootInfo};
#[cfg(test)]
entry_point!(test_kmain);
#[cfg(test)]
#[no_mangle]
pub fn test_kmain(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    halt()
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt()
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    halt()
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
