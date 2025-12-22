#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
#[allow(unused_imports)]
use core::panic::PanicInfo;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::vec;
use bootloader::{BootInfo, entry_point};
use floof::memory::{BootInfoFrameAllocator, EmptyFrameAllocator};
use floof::task::Task;
use floof::task::executor::Executor;
use floof::task::keyboard::print_keypresses;
use floof::{QemuExitCode, Testable, allocator, exit_qemu, memory, print, println, serial_println};
use floof::vga_buffer::{Color, vga_color};
use x86_64::VirtAddr;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, Page, PageTable, Translate};

macro_rules! log {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        serial_println!($($arg)*);
    }};
}

async fn six_seven() -> u32 {
    67
}

async fn example_task() {
    let number = six_seven().await;
    println!("the funny number is {}", number);
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

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    //
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) }
    // let addrs = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];
    //
    // for &addr in &addrs {
    //     let virt = VirtAddr::new(addr);
    //     let phys = mapper.translate_addr(virt);
    //     println!("{virt:?} -> {phys:?}");
    // }
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();

     #[cfg(test)]
     test_main();
    //
    // println!("did not crash!");
    // floof::hlt_loop();
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
