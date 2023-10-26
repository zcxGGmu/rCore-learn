//!An easy file system isolated from the kernel
#![no_std]
#![deny(missing_doc)]
extern crate alloc;
mod block_dev;
mod block_cache;

use block_dev::BlockDevice;
use block_cache::{
    get_block_cache,
    block_cache_sync_all,
};

pub const BLOCK_SZ: usize = 512;


