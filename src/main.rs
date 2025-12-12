#![no_std]
#![no_main]
mod vga_buffer;

use core::{f32, panic::PanicInfo};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let pi = f32::consts::PI;
    for _ in 0..20 {
        println!("the number of PI is {}", pi);
    }
    loop {}
}
