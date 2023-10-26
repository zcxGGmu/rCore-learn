//!An easy file system isolated from the kernel
#![no_std]
#![deny(missing_doc)]
extern crate alloc;
mod block_dev;
