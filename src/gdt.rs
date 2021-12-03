use x86_64::instructions::segmentation::{Segment, CS, DS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{
    Descriptor, DescriptorFlags, GlobalDescriptorTable, SegmentSelector,
};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const STACK_SIZE: usize = 0x2048;
pub static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
pub static mut PRIVATE_TSS_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss.privilege_stack_table[0] = {
            let stack_start = VirtAddr::from_ptr(unsafe { &PRIVATE_TSS_STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_data_flags =
            DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::WRITABLE;
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let data_selector = gdt.add_entry(Descriptor::UserSegment(kernel_data_flags.bits()));
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());
        (
            gdt,
            Selectors {
                code_selector,
                data_selector,
                tss_selector,
                user_data_selector,
                user_code_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    let stack = unsafe { &STACK as *const _ };
    let user_stack = unsafe { &PRIVATE_TSS_STACK as *const _ };
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector)
    }
}
