use x86_64::{instructions::interrupts::int3, structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}};
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

#[test_case]
fn breakpoint_exception() {
    int3(); //interupt
}
