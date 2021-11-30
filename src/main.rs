#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ost::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use alloc::boxed::Box;
use bootloader::entry_point;
use bootloader::BootInfo;
use core::panic::PanicInfo;
use ost::allocator::init_heap;
use ost::halt;
use ost::memory;
use ost::memory::BootInfoFrameAllocator;
use ost::println;
use ost::task::executor::Executor;
use ost::task::executor::Spawner;
use ost::task::keyboard::print_keypresses;
use ost::task::Task;
use ost::thread::context_switch_test;
use x86_64::VirtAddr;

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

entry_point!(kmain);

#[no_mangle]
fn kmain(boot_info: &'static BootInfo) -> ! {
    ost::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    init_heap(&mut mapper, &mut frame_allocator).expect("Failed to initialize heap");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    let mut spawner = Spawner::new(&executor);
    let mut task_spawner = Box::leak(Box::new(Spawner::new(&executor)));
    spawner.spawn(Task::new(example_task()));
    spawner.spawn(Task::new(print_keypresses()));
    spawner.spawn(Task::new(spawning(task_spawner)));
    context_switch_test();
    executor.run();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

async fn spawning(spawner: &mut Spawner) {
    for i in 0..10 {
        spawner.spawn(Task::new(example_task()));
        println!("Spawned task n{}", i);
    }
}
