use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{PhysAddr, VirtAddr, registers::control::Cr3, structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB}};

pub struct EmptyFrameAllocator;
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }

    pub fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_frame_number..r.range.end_frame_number);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level4_table(offset);
        OffsetPageTable::new(level_4_table, offset)
    }
}

/// MUST BE CALLED ONLY ONCE
unsafe fn active_level4_table(offset: VirtAddr) -> &'static mut PageTable {
    let (l4_frame, _) = Cr3::read();
    let phys = l4_frame.start_address();
    let virt = offset + phys.as_u64();
    let table: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *table }
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000)); // vga
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let res = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    res.expect("map_to failed").flush();
}
