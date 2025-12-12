#![no_std]
#![no_main]
mod vga_buffer;

use core::{f32, fmt::{write, Write}, panic::PanicInfo};

use crate::vga_buffer::{Buffer, Color, ColorCode, WRITER, Writer};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let pi = f32::consts::PI;
    let mut writer = WRITER.lock();
    for _ in 0..20 {
        writeln!(writer, "the number of PI is {}", pi).unwrap();
    }
    loop {}
}
