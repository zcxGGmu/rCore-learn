//! Implementation of PageTable/PageTableEntry
use super::{
    get_current, get_end, print_allocator_vec,
    frame_alloc,
    StepByOne,
    FrameTracker, PhysPageNum, PhysAddr,
    VirtAddr, VirtPageNum
};
use alloc::vec;
use alloc::vec::Vec;
use bitflags::*;
use alloc::string::String;

// PTE
const PTE_PPN_SHIFT: usize      = 10;
const PTE_PPN_BITS: usize       = 44;

// SATP
const SATP_PPN_BITS: usize      = 44;
const SATP_PPN_SHIFT: usize     = 0;
const SATP_ASID_BITS: usize     = 16;
const SATP_ASID_SHIFT: usize    = 44;
const SATP_MODE_BITS: usize     = 4;
const SATP_MODE_SHIFT: usize    = 60;
const SATP_MODE_DISABLE_MMU: usize  = 0;
const SATP_MODE_ENABLE_MMU: usize   = 8;

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
        
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
/// page table entry
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }

    pub fn writeable(&self) -> bool { 
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }

    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// 3L PageTable
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        Self {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }

    /*
       temporarily used to get contents from any user space
       from_token/translate/translate_va usually used together.
    */
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }
    
    // pte will not be created even if it is not found
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    // if the vpn mapped pte does not exist, try to create it
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        //print_allocator_vec();
        //println!("hello");
        let pte = self.find_pte_create(vpn).unwrap();
        //let ppn_pte: usize = PhysAddr::from(&(*pte) as usize).floor().into();
        //println!("ppn_pte = {}", ppn_pte);
        //print_allocator_vec();
        //println!("hello");
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    /// get satp value
    pub fn token(&self) -> usize {
        SATP_MODE_ENABLE_MMU << SATP_MODE_SHIFT | self.root_ppn.0
    }

    /// vpn => pte
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    } 

    /// va => pa 
    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }
}

/// translate a pointer to a mutable u8 Vec through page_table
pub fn translated_byte_buffer(
    token: usize,
    ptr: *const u8,
    len: usize,
) -> Vec<&'static mut [u8]> {
    let page_table_tmp = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    
    let mut v = Vec::new(); // the vector 
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table_tmp
            .translate(vpn)
            .unwrap()
            .ppn();
        
        // copy by per page
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            // end aligned to page
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]); 
        } else {
            // end is not aligned to page
            v.push(&mut ppn.get_bytes_array()
                            [start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

pub fn translated_str(token: usize, ptr: *const u8) -> String {
   let page_table = PageTable::from_token(token);
   let mut string = String::new();
   let mut va = ptr as usize;
   loop {
       let ch: u8 = *(page_table
            .translate_va(VirtAddr::from(va))
            .unwrap()
            .get_mut());
       if ch == 0 {
           break;
       } else {
           string.push(ch as char);
           va += 1;
       }
   }
   string
}

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    page_table
        .translate_va(VirtAddr::from(va))
        .unwrap()
        .get_mut()
}
