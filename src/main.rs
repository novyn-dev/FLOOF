#![no_std]
#![no_main]
mod vga_buffer;

use core::{f32, fmt::{write, Write}, panic::PanicInfo};

use crate::vga_buffer::{Buffer, Color, ColorCode, Writer};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut writer = Writer {
        col_pos: 0,
        color_code: ColorCode::new(Color::LightBlue, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    };

    let pi = f32::consts::PI;
    for _ in 0..20 {
        writeln!(writer, "the number of PI is {}", pi).unwrap();
    }
    loop {}
}
