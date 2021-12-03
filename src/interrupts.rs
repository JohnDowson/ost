use crate::{gdt, halt, print, println, serial_print, serial_println, task::clock::tick};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{
    instructions::port::Port,
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt);
        idt.page_fault.set_handler_fn(page_fault);
        idt.overflow.set_handler_fn(overflow);
        idt[InterruptIndex::RTC.as_usize()].set_handler_fn(rtc_interrupt);
        idt
    };
}

extern "x86-interrupt" fn overflow(stack_frame: InterruptStackFrame) {
    println!("Overflow\n{:#?}", stack_frame)
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard = PIC1_OFFSET + 1,
    RTC = PIC1_OFFSET + 8,
}

impl InterruptIndex {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame)
}
extern "x86-interrupt" fn double_fault(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    println!(
        "EXCEPTION: DOUBLE FAULT #{}\n{:#?}",
        error_code, stack_frame
    );
    halt()
}

extern "x86-interrupt" fn page_fault(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    println!(
        "EXCEPTION: PAGE FAULT @ {:?}\nError code: {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
    halt()
}

extern "x86-interrupt" fn timer_interrupt(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn rtc_interrupt(_stack_frame: InterruptStackFrame) {
    unsafe {
        Port::new(0x70).write(0x8Cu8);
        let u: u8 = Port::new(0x71).read();
        if (u & (1 << 4 - 1)) == 0 {
            tick();
        }
        unsafe {
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::RTC.as_u8());
        }
    }
}
