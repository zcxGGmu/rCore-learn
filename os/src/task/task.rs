//! Implementation of TaskControlBlock

use super::TaskContext;
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use crate::mm::{
    MemorySet, PhysPageNum, KERNEL_SPACE,
    MapPermission, VirtAddr,
};
use crate::trap::{trap_handler, TrapContext};

use super::{pid_alloc, KernelStack, PidHandle};

use crate::sync::UPSafeCell;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

#[derive(Copy, Clone, PartialEq)]
/// task status: Ready/Running/Zombie
pub enum TaskStatus {
    Ready,
    Running,
    Zombie,
}

/// task control core structure
pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub trap_cx_ppn: PhysPageNum,
    pub memory_set: MemorySet,
    pub base_size: usize,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }

    fn get_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    /// get inner of TaskControlBlock
    pub fn inner_exclusive_access(&self) 
        -> RefMut<'_, TaskControlBlockInner>
    {
        self.inner.exclusive_access()
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }

    /// create a new task control block
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        let (memory_set, user_sp, entry_point)
                            = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;

        /*
            Build the kernel stack while creating the process
            control block for the application and insert it
            at the high 256GiB of the kernel address space
            (after Trampoline).
        */
        let (kernel_stack_bottom, kernel_stack_top)
                            = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W
        );

        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };

        // prepare initial TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize
        );
        task_control_block
    }

    pub fn fork(&self) -> Arc<TaskControlBlock> {
        //TODO
    }

    pub fn exec(&self, elf_data: &[u8]) {
        //TODO
    }
}
