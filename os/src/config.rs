//! Configure Params

#[allow(unused)]

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;

pub const PAGE_SIZE: usize = 0x1000; //4096
pub const PAGE_SIZE_BITS: usize = 0xc; //12
pub const PTE_SIZE: usize = 8;
pub const PTE_NUM_PER_FRAME: usize = PAGE_SIZE / PTE_SIZE;

pub const MEMORY_END: usize = 0x80800000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

/// return (bottom, top) of a kernel stack for each application
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

/*
#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
*/
pub use crate::board::CLOCK_FREQ;
