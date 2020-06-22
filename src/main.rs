#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(ost::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use ost::memory;
pub use ost::{print, println};
use x86_64::structures::paging::{Page, Size4KiB};
use x86_64::VirtAddr;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ost::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ost::test::test_panic_handler(info)
}

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    ost::init();
    println!("Hello, World!");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut _mapper = unsafe { memory::init(phys_mem_offset) };
    let mut _frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    let _page = Page::<Size4KiB>::containing_address(VirtAddr::new(0));

    ost::hlt_loop()
}
