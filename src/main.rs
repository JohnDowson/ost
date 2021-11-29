#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ost::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use ost::halt;
use ost::println;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ost::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello, World!");
    ost::init();

    #[cfg(test)]
    test_main();

    println!("Did not crash!");
    halt()
}
