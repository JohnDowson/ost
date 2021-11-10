#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]

extern crate alloc;
#[macro_use]
pub mod macros;
pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod test;
pub mod vga_buffer;
#[cfg(test)]
use core::panic::PanicInfo;

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test::test_panic_handler(info)
}

pub fn init() {
    unsafe {
        // cursor-disabling magic
        use x86_64::instructions::port::Port;
        let mut port1 = Port::new(0x3d4);
        port1.write(0b00001010_u8);
        let mut port2 = Port::new(0x3d5);
        port2.write(0b00100000_u8)
    }
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
