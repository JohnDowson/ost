use super::*;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(handlers::breakpoint);
        unsafe {
            idt.double_fault
                .set_handler_fn(handlers::double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(handlers::page_fault);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(handlers::timer_interrupt);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(handlers::keyboard_interrupt);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}