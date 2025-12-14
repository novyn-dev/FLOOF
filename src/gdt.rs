use lazy_static::lazy_static;
use x86_64::{VirtAddr, structures::{gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector}, tss::TaskStateSegment}};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 4 kb
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // no proper memory management yet, just use an array

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            #[allow(clippy::let_and_return)]
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        tss
    };
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}
