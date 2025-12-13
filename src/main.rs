#![no_std]
#![no_main]

#[allow(unused_imports)]
use core::panic::PanicInfo;
use floof::{print, println};
use floof::vga_buffer::{Color, vga_color};

#[allow(clippy::empty_loop)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print!("Make yourself at home - ");
    vga_color(Color::Yellow, Color::Black);
    println!("Novyn");
    vga_color(Color::White, Color::Black);

    loop {}
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
    use floof::test_panic_handler;
    test_panic_handler(info);
}
