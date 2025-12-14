#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;

use core::panic::PanicInfo;
use x86_64::{instructions::tables::load_tss, registers::segmentation::{CS, Segment}};

use crate::{gdt::GDT, interrupts::init_idt};

/// combines both println! and serial_println!
macro_rules! log {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        serial_println!($($arg)*);
    }};
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    log!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // 0xf4 is the port to exit
        port.write(code as u32);
    }
}

pub fn init() {
    init_idt();

    let selectors = &GDT.1;

    GDT.0.load();
    unsafe {
        CS::set_reg(selectors.code_selector);
        load_tss(selectors.tss_selector);
    }
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() {
    init();
    test_main();
    loop {}
}

#[test_case]
fn it_works() {
    let sum = 1 + 1;
    assert_eq!(sum, 2);
}

