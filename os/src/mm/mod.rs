//! Memory Management Subsystem

mod heap_allocator;
mod address;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

/// initiate heap_allocator...
pub fn init() {
    heap_allocator::init_heap();
    heap_allocator::heap_test();
}
