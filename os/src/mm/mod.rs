//! Memory Management Subsystem

mod heap_allocator;
mod address;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use page_table::{PageTableEntry};

/// initiate heap_allocator...
pub fn init() {
    heap_allocator::init_heap();
    heap_allocator::heap_test();
}
