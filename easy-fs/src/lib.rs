//!An easy file system isolated from the kernel
#![no_std]
#![deny(missing_doc)]
extern crate alloc;
mod block_dev;
mod block_cache;
mod layout;

use block_dev::BlockDevice;
use block_cache::{
    get_block_cache,
    block_cache_sync_all,
};
use layout::*;

pub const BLOCK_SZ: usize = 512;


