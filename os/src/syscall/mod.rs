// syscall ID
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

mod fs;
mod process;

use fs::{*};
use process::{*};

pub fn syscalls(syscall_id: usize, args: [usize; 3]) -> isize {
   match sycall_id {
       SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
       SYSCALL_EXIT => sys_exit(args[0] as i32),
       _ => panic!("Current rCore unsupported syscall_id: {}", syscall_id),
   } 
}
