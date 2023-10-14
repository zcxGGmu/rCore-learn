//! Types related to task management

use super::TaskContext;
use crate::config::{TRAP_CONTEXT, kernel_stack_position};
use crate::mm::{
    MemorySet, PhysPageNum, KERNEL_SPACE,
    MapPermission, VirtAddr,
};
use crate::trap::{trap_handler, TrapContext};

#[derive(Copy, Clone, PartialEq)]
/// task status: Uninit/Ready/Running/Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

/// task control core structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    /// direct access contents stored in frame that trap_cx_ppn ponits
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    /// get user space satp
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
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
        let task_cx = TaskContext::
        
        let task_control_block = Self {
            task_status,
            task_cx,
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        }

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
    
}

