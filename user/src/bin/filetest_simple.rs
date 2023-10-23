#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib:: {
    open,
    close,
    read,
    write,
    OpenFlags,
};

#[no_mangle]
pub fn main() -> i32 {
    // create filea and write "Hello. World!" into it
    let test_str = "Hello, World!";
    let filea = "filea\0";
    let fd = open(filea, OpenFlags::CREATE | OpenFlags::WRONLY);
    assert!(fd > 0);
    write(fd as usize, test_str.as_bytes());
    close(fd as usize);

    // open filea and read all contents from it
    let fd = open(filea, OpenFlags::RDONLY);
    assert!(fd > 0);
    let mut buffer = [0u8; 100];
    let read_len = read(fd, &mut buffer) as usize;
    close(fd);
    
    // check
    assert_eq!(
        test_str,
        core::str::from_utf8(&buffer[..read_len].unwrap()),
    );
    println!("file_test passed!");
}
