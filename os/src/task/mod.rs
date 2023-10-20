//! Implementation of TaskManager
mod context;
mod switch;
mod pid;
mod manager;
mod processor;
#[allow(clippy::module_inception)]
mod task;

use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};
use switch::__switch;
pub use context::TaskContext;

use alloc::vec::Vec;
use alloc::sync::Arc;

use crate::loader::get_app_data_by_name;
pub use pid::{pid_alloc, KernelStack, PidAllocator, PidHandle};
pub use manager::{add_task, fetch_task, TaskManager};
pub use processor::{
    current_task, take_current_task, current_trap_cx, 
    current_user_token, run_tasks, schedule, Processor,
};


lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> =
        Arc::new(TaskControlBlock::new(get_app_data_by_name("initproc").unwrap()));
}

pub fn add_initproc() {
    add_task(INITPROC.clone());
}

/// suspend the current 'Running' task and run the next task in task_queue
pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task); 
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// exit the current 'Running' task and run the next task in task_queue
pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    inner.task_status = TaskStatus::Zombie;
    inner.exit_code = exit_code;
    
    // under initproc
    {
       let mut initproc_inner = INITPROC.inner_exclusive_access();
       for child in inner.children.iter() {
           child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
           initproc_inner.children.push(child.clone());
       }
    }

    inner.children.clear();
    inner.memory_set.recycle_data_pages();

    drop(inner);
    // drop task manually to maintain rc correctly
    drop(task);
    
    // we do not save task context
    let mut _unused =TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}
