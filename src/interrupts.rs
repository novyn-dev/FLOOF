use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::{gdt::DOUBLE_FAULT_IST_INDEX, println};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe { idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX); }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _err_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, _err_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT\n{:#?}", stack_frame);
}

// PIC offsets range from 32..47, typically
pub const PIC1_OFFSET: u8 = 32; // 32 + 8
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8; // 32 + 8 + 8

pub static PICS: Mutex<ChainedPics> = Mutex::new( unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

#[test_case]
fn breakpoint_exception() {
    use x86_64::instructions::interrupts::int3;
    int3(); //interupt
}
