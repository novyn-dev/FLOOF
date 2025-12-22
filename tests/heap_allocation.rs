#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(floof::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
#[allow(unused_imports)]
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec};
use bootloader::{BootInfo, entry_point};
use floof::{allocator::{self, HEAP_SIZE}, hlt_loop, memory::{self, BootInfoFrameAllocator}};
use x86_64::VirtAddr;

entry_point!(main);
fn main(bootinfo: &'static BootInfo) -> ! {
    let phys_mem_offset = bootinfo.physical_memory_offset;
    let mut mapper = unsafe { memory::init(VirtAddr::new(phys_mem_offset)) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&bootinfo.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Couldn't initialize heap");

    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    floof::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let x = Box::new(67);
    let y = Box::new(69);
    assert_eq!(*x, 67);
    assert_eq!(*y, 69);
}

#[test_case]
fn many_boxes() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n-1) * n / 2)
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}

