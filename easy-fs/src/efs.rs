/*!
  An easy file system on block:
  The EasyFileSystem knows where each layout area is located,
  and disk block allocation and reclamation can be completed through it,
  so it can be regarded as a disk block manager in a sense.
*/
use super::{
    block_cache_sync_all, get_block_cache,
    Bitmap, BlockDevice, DiskInode, DiskInodeType, SuperBlock,
};
use crate::BLOCK_SZ;
use alloc::sync::Arc;
use spin::Mutex;
