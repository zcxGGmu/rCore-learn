//! Process management syscalls
use crate::loader::get_app_data_by_name;
use crate::task::{
    add_task, current_task, current_user_token,
    exit_current_and_run_next, suspend_current_and_run_next,
};
use crate::mm::{
    translated_str,
    translated_byte_buffer,
    translated_refmut,
};
use log::{*};
use crate::timer::get_time_ms;
use alloc::sync::Arc;

/// task exits and sbumit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources proactively
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

/// get time in milliseconds
pub fn sys_get_timer() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap_context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    /* 
        for child process, fork returns 0. x[10] is a0 reg.
    */
    trap_cx.x[10] = 0;

    // add new task to task_schedule_queue
    add_task(new_task);
    new_pid as isize
}

// The argument passed in is the first address of the string corresponding to the application name
pub fn sys_exec(path: *const u8) -> isize {
    // current `path` is va 
    let token = current_user_token();
    let path = translated_str(token, path);
    // execute exec
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    
    // find a child process with [pid]
    let mut inner = task.inner_exclusive_access();
    if inner.children
        .iter()
        .find(|p| {pid == -1 || pid as usize == p.getpid()})
        .is_none()
    {
        return -1;
    }

    // find [pid] is zombie ?
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)|
            {
                p.inner_exclusive_access().is_zombie() &&
                (pid == -1 || pid as usize == p.getpid())
            }
        );

    // final treatment: reclaim the remaining resources of the process
    if let Some((idx, p)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // exit_code write back to user mode
        let exit_code = child.inner_exclusive_access().exit_code;
        *translated_refmut(inner.get_user_token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}
