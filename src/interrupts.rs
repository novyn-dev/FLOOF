use pc_keyboard::HandleControl;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{instructions::port::Port, structures::idt::{InterruptDescriptorTable, InterruptStackFrame}};
use lazy_static::lazy_static;
use crate::{gdt::DOUBLE_FAULT_IST_INDEX, print, println};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard // comes after the PIC lmfao
}

impl InterruptIndex {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn as_usize(&self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe { idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX); }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _err_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, err_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error code: {:?}", err_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    let interrupt_idx = InterruptIndex::Timer.as_u8(); // timer idx
    unsafe {
        PICS.lock().notify_end_of_interrupt(interrupt_idx);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{Keyboard, ScancodeSet1, layouts};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore));
    }

    let mut port = Port::new(0x60); // PS/2 I/O port
    let scancode: u8 = unsafe { port.read() };
    let mut keyboard = KEYBOARD.lock();

    #[allow(clippy::collapsible_if)]
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                pc_keyboard::DecodedKey::RawKey(key_code) => print!("{:?}", key_code),
                pc_keyboard::DecodedKey::Unicode(c) => print!("{}", c),
            }
        }
    }

    let interrupt_idx = InterruptIndex::Keyboard.as_u8(); // kb idx
    unsafe {
        PICS.lock().notify_end_of_interrupt(interrupt_idx);
    }
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
