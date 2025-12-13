#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod vga_buffer;

use core::panic::PanicInfo;

use crate::vga_buffer::{Color, change_vga_color};

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[allow(clippy::empty_loop)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print!("Make yourself at home - ");
    change_vga_color(Color::Yellow, Color::Black);
    println!("Novyn");

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn it_works() {
    let sum = 1 + 1;
    assert_eq!(sum, 2);
    println!("[ok]");
}
