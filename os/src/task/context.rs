//! Implementation of TaskContext

use crate::trap::trap_return;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    /// return address of `trap_return`
    ra: usize,
    /// kernel stack of application
    sp: usize,
    /// callee save registers: s0~s11
    s: [usize; 12],
}

impl TaskContext {
    /// init task context
    pub fn zero_init() -> Self  {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// set TaskContext
    pub fn  goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
