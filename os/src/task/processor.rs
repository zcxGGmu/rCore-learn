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

/*  task schedule  */
pub fn run_tasks() {
    //TODO
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    //TODO
}
