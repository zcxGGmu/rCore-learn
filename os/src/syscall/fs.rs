//! File and filesystem-related syscalls

use crate::mm::translated_byte_buffer;
use crate::task::current_user_token;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let token = current_user_token();
            let buffers = translated_byte_buffer(token, buf, len);
            for buf in buffers {
                print!("{}", core::str::from_utf8(buf).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Current rCore unsupported fd in sys_write!");
        }
    }
}
