use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::Cr3;
#[allow(unused_imports)]
use x86_64::structures::paging::{
    page_table::FrameError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame,
    Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

unsafe fn active_l4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (l4_table_frame, _) = Cr3::read();
    let phys = l4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    #[allow(clippy::clippy::missing_safety_doc)]
    pub unsafe fn init(memory_map: &'static MemoryMap) -> BootInfoFrameAllocator {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let address_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = address_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub fn translate_address(address: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    _translate_address(address, physical_memory_offset)
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_l4_table(physical_memory_offset);
    OffsetPageTable::new(l4_table, physical_memory_offset)
}

#[doc(hidden)]
fn _translate_address(adr: VirtAddr, pmo: VirtAddr) -> Option<PhysAddr> {
    let (l4_table_frame, _) = Cr3::read();
    let table_indexes = [
        adr.p4_index(),
        adr.p3_index(),
        adr.p2_index(),
        adr.p1_index(),
    ];
    let mut frame = l4_table_frame;
    for &index in &table_indexes {
        let virt = pmo + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge pages not supported"),
        };
    }
    Some(frame.start_address() + u64::from(adr.page_offset()))
}
