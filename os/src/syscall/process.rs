//! Process management syscalls

use log::{*};

use crate::task::{
    exit_current_and_run_next,              
    suspend_current_and_run_next
};
use crate::timer::get_time_ms;

/// task exits and sbumit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
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
