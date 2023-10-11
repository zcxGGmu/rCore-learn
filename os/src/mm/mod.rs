//! Memory Management Subsystem

mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use page_table::PageTableEntry;
pub use frame_allocator::{frame_alloc, FrameTracker};
pub use page_table::PTEFlags;

/// initiate heap_allocator/frame_allocator...
pub fn init() {
    // init heap_allocator
    heap_allocator::init_heap();
    heap_allocator::heap_test();

    // init frame_allocator
    frame_allocator::init_frame_allocator();
    frame_allocator::frame_allocator_test();
}
