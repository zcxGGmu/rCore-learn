//! Memory Management Subsystem

mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
mod memory_set;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, StepByOne};

pub use frame_allocator::{
    frame_alloc, FrameTracker,
    get_current, get_end, print_allocator_vec
};

use page_table::PTEFlags;
pub use page_table::{
    PageTable, PageTableEntry,
    translated_byte_buffer,
    translated_str,
    translated_refmut,
};

pub use memory_set::{MapPermission, MemorySet, KERNEL_SPACE};
pub use memory_set::remap_test;

/// initiate heap_allocator/frame_allocator...
pub fn init() {
    // init heap_allocator
    heap_allocator::init_heap();
    //heap_allocator::heap_test();

    // init frame_allocator
    frame_allocator::init_frame_allocator();
    //frame_allocator::frame_allocator_test();

    // enable mmu
    KERNEL_SPACE.exclusive_access().activate();
    print_allocator_vec();
}
