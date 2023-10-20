//!Implementation of [`Processor`] and add a idle control flow
use super::__switch;
use super::{TaskStatus, fetch_task};
use super::{TaskContext, TaskControlBlock};
use crate::trap::TrapContext;
use crate::sync::UPSafeCell;

use alloc::sync::Arc;
use lazy_static::*;

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> =
        unsafe { UPSafeCell::new(Processor::new()) };
}

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
   pub fn new() -> Self {
       Processor {
           current: None,
           idle_task_cx: TaskContext::zero_init(),
       }
   }
   
   pub fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
       &mut self.idle_task_cx as *mut _
   }

   // remove self.current
   pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
       self.current.take()
   }

   pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
       self.current.as_ref().map(Arc::clone)
   }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

/*  task scheduling  */
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            
            processor.current = Some(task);
            drop(processor);

            // swicth to next_task
            unsafe {
                __switch(
                    idle_task_cx_ptr,
                    next_task_cx_ptr
                );
            };
        }
    }
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);

    /* 
        switch to idle control flow,
        the specific scheduling details are not concerned here.
    */
    unsafe {
        __switch(
            switched_task_cx_ptr,
            idle_task_cx_ptr,
        );
    }
}
