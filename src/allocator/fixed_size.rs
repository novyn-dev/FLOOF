use core::{alloc::{GlobalAlloc, Layout}, ptr::{NonNull, null_mut}};

use crate::allocator::Locked;

/// alignments must be power of 2 (binary)
/// ranging from 16 to 2048
const BLOCK_SIZES: &[usize] = &[16, 32, 64, 128, 256, 512, 1024, 2048];

#[repr(C)]
struct ListNode {
    next: Option<*mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<*mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap
}

unsafe impl Send for FixedSizeBlockAllocator {}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<*mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// this function is unsafe because the caller must guarantee that the given heap bounds are
    /// valid, and the address is unused 
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe { self.fallback_allocator.init(heap_start, heap_size); }
    }

    fn fallback_alloc(&mut self, layout: &Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(*layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => null_mut(),
        }
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        // find suitable size
        match list_index(&layout) {
            Some(size_idx) => {
                match allocator.list_heads[size_idx] {
                    Some(node) => {
                        unsafe { allocator.list_heads[size_idx] = (*node).next.take() };
                        node as *mut u8
                    }
                    None => {
                        let size = BLOCK_SIZES[size_idx];
                        let layout = Layout::from_size_align(size, size).unwrap();
                        allocator.fallback_alloc(&layout)
                    },
                }
            }
            None => allocator.fallback_alloc(&layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        // find suitable size
        match list_index(&layout) {
            Some(size_idx) => {
                let size = BLOCK_SIZES[size_idx];
                let new_node = ListNode {
                    next: allocator.list_heads[size_idx],
                };
                assert!(size_of::<ListNode>() <= size);
                assert!(align_of::<ListNode>() <= size);

                let ptr = ptr as *mut ListNode;
                unsafe {
                    ptr.write(new_node);
                    allocator.list_heads[size_idx] = Some(ptr);
                }
            }
            None => {
                let ptr = NonNull::new(ptr).expect("Null pointer");
                unsafe { allocator.fallback_allocator.deallocate(ptr, layout) };
            },
        }
    }
}

/// helper function to get appropriate block size
fn list_index(layout: &Layout) -> Option<usize> {
    // choose between layout.size() or layout.align() as the required block size
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size) // return the suitable size
}
