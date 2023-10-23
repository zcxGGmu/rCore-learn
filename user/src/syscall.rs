use core::arch::asm;

const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm! {
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        };
    }
    ret
}

pub fn sys_open(path: &str, flags: u32) -> isize {
    syscall(SYSCALL_OPEN, [path.as_ptr() as usize, flags as usize, 0])
}

pub fn sys_close(fd: usize) -> isize {
    syscall(SYSCALL_CLOSE, [fd, 0, 0])
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
   syscall(
       SYSCALL_READ,
       [fd, buffer.as_mut_ptr() as usize, buffer.len()]
    )
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0])
}

/**
    Function: The current process forks out a child process.
    Return value: 0 for child processes and PID for current processes.
    syscall ID：220
*/ 
pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0])
}

/**
    Function: Empty the address space of the current process and load a specific executable file,
              return to user mode and start its execution.
    
    Parameter: path gives the name of the executable to be loaded;
    
    Return value: -1 if an error occurs (such as not finding an executable with a matching name),
                  otherwise it should not be returned.
    
    syscall ID：221
*/
pub fn sys_exec(path: &str) -> isize {
    syscall(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

/**
    Function: The current process waits for a child process to become a zombie process,
              reclaims all its resources and collects its return value.
    
    Parameter: pid indicates the process ID of the child process to wait, if -1 means
               waiting for any child process;
    
    exit_code: indicates the address where the return value of the child process is saved,
              and if this address is 0, it does not have to be saved.
    
    Return value: -1 if the child process to wait does not exist; otherwise -2 if none of
                  the child processes to wait for have ended; Otherwise, the process ID
                  of the ending child process is returned.

    syscall ID：260
*/
pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    syscall(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}
