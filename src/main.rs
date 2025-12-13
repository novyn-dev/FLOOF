#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod vga_buffer;
mod serial;

use core::panic::PanicInfo;
use crate::vga_buffer::{Color, vga_color};

/// combines both println! and serial_println!
macro_rules! log {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        serial_println!($($arg)*);
    }};
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

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        vga_color(Color::LightGreen, Color::Black);
        serial_println!("[ok]");
        vga_color(Color::White, Color::Black);
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    log!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[allow(clippy::empty_loop)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print!("Make yourself at home - ");
    vga_color(Color::Yellow, Color::Black);
    println!("Novyn");
    vga_color(Color::White, Color::Black);

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn it_works() {
    let sum = 1 + 1;
    assert_eq!(sum, 2);
}
