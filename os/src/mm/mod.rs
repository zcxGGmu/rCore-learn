//! Memory Management Subsystem

mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
mod memory_set;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, StepByOne};

pub use frame_allocator::{frame_alloc, FrameTracker};

use page_table::PTEFlags;
pub use page_table::{
    PageTable, PageTableEntry,
};

/// initiate heap_allocator/frame_allocator...
pub fn init() {
    // init heap_allocator
    heap_allocator::init_heap();
    heap_allocator::heap_test();

    // init frame_allocator
    frame_allocator::init_frame_allocator();
    frame_allocator::frame_allocator_test();
}
