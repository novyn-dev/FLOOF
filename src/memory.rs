use x86_64::{VirtAddr, structures::paging::PageTable};

pub unsafe fn active_level4_table(offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (l4_table, _) = Cr3::read();

    let phys = l4_table.start_address();
    let virt = offset + phys.as_u64();
    let table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *table_ptr }
}
