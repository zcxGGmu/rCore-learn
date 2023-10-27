//!Rom layout
//![super_block][inode_bitmap][inode_area][data_bitmap][data_area]
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Debug, Formatter, Result};

/// magic number for sanity check
const EFS_MAGIC: u32 = 0x3b800001;
/// disk_inode <=> data_block
/** [INODE_*_COUNT]:
    The number of actual file data blocks represented by direct index and indirect index;
    [*_BOUND]:
    The index value in the block space pointed to by 'DiskNode'
    is represented as the index block of the file
*/
const INODE_DIRECT_COUNT: usize = 28;
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;
/// the max length of inode name
const NAME_LENGTH_LIMIT: usize = 27;

/// super_block
#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl SuperBlock {
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ) {
        *self = Self {
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("SuperBlock")
            .field("total_blocks", &self.total_blocks)
            .field("inode_bitmap_blocks", &self.inode_bitmap_blocks)
            .field("inode_area_blocks", &self.inode_area_blocks)
            .field("data_bitmap_blocks", &self.data_bitmap_blocks)
            .field("data_area_blocks", &self.data_area_blocks)
            .finish()
    }
}


/// type of disk_inode => {File/Directory}
#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Directory,
}

type IndirectBlock = [u32; BLOCK_SZ / 4];
type DataBlock = [u8; BLOCK_SZ];

/// struct disk_inode
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}

impl DiskInode {
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }

    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }

    pub fn data_blocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }

    fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32 
    }

    /// return total number of blocks needed include indirect1/2
    /** 
        The data block area contains not only the file data,
        but also the index information of the file data block in some cases.
    */
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;
        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1;
        }
        // indirect2 => 1 * indirect2 + n * indirect1
        if data_blocks > INDIRECT1_BOUND {
            total += 1;
            total += (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1)
                        / INODE_INDIRECT1_COUNT;
        }
        total as u32
    }

    /// get the number of data blocks that have to be allocated 
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size as u32);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }

    /// get global block_id given inner_id
    /// inner_id => inner id of disk_inode pointed to file data_block area. [0.._data_blocks(size)]
    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            get_block_cache(self.indirect1 as usize, Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[inner_id - INODE_DIRECT_COUNT]
                })
        } else {
            let last = inner_id - INDIRECT1_BOUND;
            // find indirect1
            let indirect1 = get_block_cache(self.indirect2 as usize, Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[last / INODE_INDIRECT1_COUNT]
                });
            get_block_cache(indirect1 as usize, Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[last % INODE_INDIRECT1_COUNT]
                })
        }
    }

    /// increase the size of current disk_inode
    /**
        new_size: when writing data to a file, new_size =  old_size + write_data_len
        new_blocks: increase_size is only responsible for maintaining the relationship 
                    between block numbers and indexes in the disk_inode, so the available 
                    blocks need to be allocated in advance and passed as parameters before 
                    calling this function
    */ 
    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Arc<dyn BlockDevice>,
    ) {
       // these blocks is used to store file data
       let mut current_blocks = self.data_blocks();
       self.size = new_size;
       let mut total_blocks = self.data_blocks();
       let mut new_blocks = new_blocks.into_iter();
       // fill direct
       while current_blocks < total_blocks.min(INODE_DIRECT_COUNT as u32) {
           self.direct[current_blocks as usize] = new_blocks.next().unwrap();
           current_blocks += 1;
       }
       // alloc indirect1
       if total_blocks > INODE_DIRECT_COUNT {
           if current_blocks == INODE_DIRECT_COUNT as u32 {
               self.indirect1 = new_blocks.next().unwrap();
           }
           current_blocks -= INODE_DIRECT_COUNT as u32;
           total_blocks -= INODE_DIRECT_COUNT as u32;
       } else {
           return;
       }
       // fill indirect1
       get_block_cache(self.indirect1 as usize, Arc::clone(block_device))
           .lock()
           .modify(0, |indirect1: &mut IndirectBlock| {
               while current_blocks < total_blocks.min(INODE_INDIRECT1_COUNT as u32) {
                   indirect1[current_blocks as usize] = new_blocks.next().unwrap();
                   current_blocks += 1;
               }
           });
       // alloc indirect2
       if total_blocks > INODE_INDIRECT1_COUNT as u32 {
           if current_blocks == INODE_INDIRECT1_COUNT as u32 {
               self.indirect2 = new_blocks.next().unwrap();
               
           }
           current_blocks -= INODE_INDIRECT1_COUNT as u32;
           total_blocks -= INODE_INDIRECT1_COUNT as u32;
       } else {
           return;
       }
       // fill indirect2
       let mut a0 = current_blocks as usize / INODE_INDIRECT1_COUNT;
       let mut b0 = current_blocks as usize % INODE_INDIRECT1_COUNT;
       let a1 = total_blocks as usize / INODE_INDIRECT1_COUNT;
       let b1 = total_blocks as usize % INODE_INDIRECT1_COUNT;
       get_block_cache(self.indirect2 as usize, Arc::clone(block_device))
           .lock()
           .modify(0, |indirect2: &mut IndirectBlock| {
               while (a0 < a1) || (a0 == a1 && b0 < b1) {
                   if b0 == 0 {
                       indirect2[a0] = new_blocks.next().unwrap();
                   }
                   get_block_cache(indirect2[a0] as usize, Arc::clone(block_device))
                       .lock()
                       .modify(0, |indirect1: &mut IndirectBlock| {
                          indirect1[b0] = new_blocks.next().unwrap(); 
                       });
                   // move to next
                   b0 += 1;
                   if b0 == INODE_INDIRECT1_COUNT {
                       b0 = 0;
                       a0 += 1;
                   }
               }
           });
    }

    /// clear size to zero and return blocks that should be deallocated
    /// we will clear the block contents to zero later
    pub fn clear_size(&mut self, block_device: &Arc<dyn BlockDevice>) -> Vec<u32> {
        //TODO
    }

    /// read data from current disk_inode
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        //TODO
    }

    /// write data into current disk_inode
    /// size must be adjusted properly beforehand
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        //TODO
    }
}











