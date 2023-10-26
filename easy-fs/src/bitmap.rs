//!Bitmap => {inode_bitmap/data_bitmap}
/*!
  Bitmap layout:
                /  block_0  /  block_1  /  block_2  /
                /u64u64../
              => [u64; 64]
*/
use super::{
    get_block_cache,
    BlockDevice,
    BLOCK_SZ,
};

use alloc::sync::Arc;

/// bimap_block
type BitmapBlock = [u64; 64];
/// number of bits in a block
const BLOCK_BITS: usize = BLOCK_SZ * 8;

/// struct Bitmap => it may contains many blocks
/// Bitmap is used to allocate or reclaim blocks
pub struct Bitmap {
    start_block_id: usize,
    //number of blocks
    blocks: usize,
}

/// decompose bitmap_area_inner_bit into (block_pos, bits64_pos, inner_pos)
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS;
    let bits64_pos = (bit % BLOCK_BITS) / 64;
    let inner_pos = (bit % BLOCK_BITS) % 64;
    (block_pos, bits64_pos, inner_pos)
}

impl Bitmap {
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }
 
    /// allocate a new {inode/data}_block from block device 
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(
                block_id + self.start_block_id as usize,
                Arc::clone(block_device),
            ).lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    .map(|(bits64_pos, bits64)|
                         (bits64_pos, bits64.trailing_ones() as usize))
                {
                    // modify cache
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    // returns the offset bit relative to the entire bitmap
                    Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize)
                } else {
                    None
                }
            });
            if pos.is_some() {
                return pos;
            }
        }
        None
    }

    /// deallocate a {inode/data}_block
    /// the passed parameter `bit` is the offset relative to the entire bitmap
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(block_pos + self.start_block_id, Arc::clone(block_device))
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
                bitmap_block[bits64_pos] -= 1u64 << inner_pos;
            });
    }

    /// get max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}
