#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

#[allow(unused_imports)]
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use floof::{QemuExitCode, Testable, exit_qemu, print, println, serial_println};
use floof::vga_buffer::{Color, vga_color};
use x86_64::registers::control::Cr3;

macro_rules! log {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        serial_println!($($arg)*);
    }};
}

fn test_runner(tests: &[&dyn Testable]) {
    log!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

// no_mangle isnt needed
entry_point!(kernel_entry);
fn kernel_entry(boot_info: &'static BootInfo) -> ! {
    print!("Make yourself at home - ");
    vga_color(Color::Yellow, Color::Black);
    println!("Novyn");
    vga_color(Color::White, Color::Black);

    floof::init();

    #[cfg(test)]
    test_main();

    println!("did not crash!");
    floof::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    floof::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use floof::test_panic_handler;
    test_panic_handler(info);
}
