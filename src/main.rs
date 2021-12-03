#![allow(clippy::empty_loop)]
#![no_std]
#![no_main]
#![feature(box_syntax)]
#![feature(custom_test_frameworks)]
#![feature(asm_const)]
#![feature(asm)]
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
use ost::serial_println;
use ost::task::clock::print_time;
use ost::task::executor::Executor;
use ost::task::executor::Spawner;
use ost::task::keyboard::print_keypresses;
use ost::task::Task;
use x86_64::instructions::port::Port;
use x86_64::software_interrupt;
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

// unsafe fn virt_to_physical<'m>(
//     virt: VirtAddr,
//     phys_mem_offset: u64,
//     mapper: &'m mut OffsetPageTable,
// ) -> Option<(PhysAddr, &'m PageTableEntry)> {
//     // l4
//     let pte = &mapper.level_4_table()[virt.p4_index()];
//     let pte_present = pte.flags().contains(PageTableFlags::PRESENT);
//     if !pte_present {
//         return None;
//     }
//     // l3
//     let next_pt = get_next_pt(pte, phys_mem_offset);
//     let pte = &next_pt[virt.p3_index()];
//     let pte_present = pte.flags().contains(PageTableFlags::PRESENT);
//     let pte_huge = pte.flags().contains(PageTableFlags::HUGE_PAGE);
//     if !pte_present {
//         return None;
//     } else if pte_huge {
//         let offset = virt.as_u64() & 0x3fffffff;
//         return Some((pte.addr() + offset, pte));
//     }
//     // l2
//     let next_pt = get_next_pt(pte, phys_mem_offset);
//     let pte = &next_pt[virt.p2_index()];
//     let pte_present = pte.flags().contains(PageTableFlags::PRESENT);
//     let pte_huge = pte.flags().contains(PageTableFlags::HUGE_PAGE);
//     if !pte_present {
//         return None;
//     } else if pte_huge {
//         let offset = virt.as_u64() & 0x1fffff;
//         return Some((pte.addr() + offset, pte));
//     }
//     // l1
//     let next_pt = get_next_pt(pte, phys_mem_offset);
//     let pte = &next_pt[virt.p1_index()];
//     let pte_present = pte.flags().contains(PageTableFlags::PRESENT);
//     if !pte_present {
//         return None;
//     } else {
//         let offset = virt.as_u64() & 0xfff;
//         return Some((pte.addr() + offset, pte));
//     }
// }

// unsafe fn get_next_pt(pte: &PageTableEntry, phys_mem_offset: u64) -> &mut PageTable {
//     &mut *((pte.addr() + phys_mem_offset).as_u64() as *mut PageTable)
// }

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
    let task_spawner = Box::leak(box Spawner::new(&executor));
    //spawner.spawn(Task::new(example_task()));
    spawner.spawn(Task::new(print_keypresses()));
    spawner.spawn(Task::new(print_time()));
    //spawner.spawn(Task::new(spawning(task_spawner)));

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
