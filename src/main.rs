#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
mod vga_buffer;

use core::panic::PanicInfo;

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
    println!("Make yourself at home - Novyn");

    loop {}
}
