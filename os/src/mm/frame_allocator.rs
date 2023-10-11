//! Implementation of FrameAllocator
//! available physical page => [ekernel, MEMORY_END)
/*!
  We can categorize the content stored in physical page frames into two types:
  1. Application/Kernel data and code.
  2. Application/Kernel multi-level page tables.
*/

use super::{PhysAddr, PhysPageNum};
use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use core::fmt::{self, Debug, Formatter};
use lazy_static::*;

/// RAII: use a tracker to manage the lifetime of frame
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
       //page cleaning
       let bytes_array = ppn.get_bytes_array();
       for i in bytes_array {
           *i = 0;
       }
       Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
       frame_dealloc(self.ppn); 
    }
} 

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// an simple implementation for frame allocator
pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    
    // a container that recycles physical pages
    recycled: Vec<usize>,
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else if self.current == self.end {
            None
        } else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }
    
    fn dealloc(&mut self, ppn: PhysPageNum) {
       let ppn = ppn.0;
       if ppn >= self.current || self.recycled
           .iter()
           .find(|&v| {*v == ppn})
           .is_some() {
               panic!("Frame ppn={:#x} has not been allocated!", ppn);
       }
       self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    /// global frame allocator instance
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe {
            UPSafeCell::new(FrameAllocatorImpl::new())
        };
}

/// initiate the frame allocator using [ekernel, MEMORY_END)
pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor(),
    );
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

/// deallocate a frame
pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .dealloc(ppn);
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
