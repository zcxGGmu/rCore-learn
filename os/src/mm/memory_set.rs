/*
    Implementation of address space abstrct,
    it should contains some levels:
    MemorySet => Kernel/Application
    +-> [MapArea_0][MapArea_1] ... [MapArea_n]
        these logical segments are related to each other,
        but not necessarily contiguous.
        +-> vpn_range => [start, end]
            We describe a contiguous segment of virtual 
            memory by a logical segment `MapArea`.
            All virtual pages in the vpn_range is eventually 
            mapped onto the physical page frame.
*/

use super::{frame_alloc, FrameTracker};
use super::{PTEFlags, PageTable, PageTableEntry};
use super::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::{
    PAGE_SIZE, MEMORY_END, TRAMPOLINE,
    USER_STACK_SIZE, TRAP_CONTEXT
};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use bitflags::*; 
use lazy_static::*;
use alloc::sync::Arc;
use crate::sync::UPSafeCell;
use core::arch::asm;
use riscv::register::satp;

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe {
            UPSafeCell::new(MemorySet::new_kernel())
    });
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

/// MapArea => contains many virtual pages
pub struct MapArea {
    vpn_range: VPNRange,
    data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        map_perm: MapPermission
    ) -> Self {
        let start_vpn = start_va.floor();
        let end_vpn = end_va.ceil();
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }

    pub fn from_another(another: &MapArea) -> Self {
        Self {
            vpn_range: VPNRange::new(
                another.vpn_range.get_start(),
                another.vpn_range.get_end()
            ),
            data_frames: BTreeMap::new(),
            map_type: another.map_type,
            map_perm: another.map_perm,
        }
    }

    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }

    // map all virtual pages onto physical page in current MapAreas
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    // unmap all virtual pages onto physical page in current MapAreas
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }

    /*
        When calling this function, it is required that the size of 
        the data slice does not exceed the total size of the current 
        logical segment, and the data in the slice will be aligned to 
        the beginning of the logical segment. Subsequently, it will be 
        copied page by page to actual physical page frames.
        
        In fact, the size of data may be shorter than PAGE_SIZE, we must
        allocate a entire page for it.
     */
    pub fn copy_data(&mut self, page_table: &mut PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let data_len = data.len();
        loop {
            let src = &data[start..data_len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= data_len {
                break;
            }
            current_vpn.step();
        }
    }
}

/// MemorySet => contains many MapAreas
pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    pub fn push(&mut self, mut map_area: MapArea,
                data: Option<&[u8]>) {
       map_area.map(&mut self.page_table);
       if let Some(data) = data {
           map_area.copy_data(&mut self.page_table, data);
       }
       self.areas.push(map_area);
    }

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission)
    {
       self.push(
           MapArea::new(start_va, end_va, MapType::Framed, permission),
           None
       ); 
    }

    /**
        Remove `MapArea` that starts with start_vpn, two things need to do:
        1) unmap `MapArea` va->pa
        2) remove kernelstack area of KERNEL_SPACE.areas
    */
    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
       if let Some((idx, area)) = self
           .areas
           .iter_mut()
           .enumerate()
           .find(|(_, area)| area.vpn_range.get_start() == start_vpn)
        {
            area.unmap(&mut self.page_table);
            self.areas.remove(idx);
        }
    }

    pub fn recycle_data_pages(&mut self) {
        self.areas.clear();
    }

    /// Mention that trampoline is not collected by areas
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X
        );
    }

    /// create kernel address space
    /// Kernel stack is not created here
    pub fn new_kernel() -> Self {
       let mut memory_set = Self::new_bare();
       
       /* high 256GiB */
       memory_set.map_trampoline();

       /* low 256GiB */
       // map kernel sections 
       println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
       println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
       println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
       println!(
           ".bss [{:#x}, {:#x})",
           sbss_with_stack as usize, ebss as usize
       );
       println!("mapping .text section");
       memory_set.push(MapArea::new(
               (stext as usize).into(),
               (etext as usize).into(),
               MapType::Identical,
               MapPermission::R | MapPermission::X
       ), None);

       println!("mapping .rodata section");
       memory_set.push(MapArea::new(
               (srodata as usize).into(),
               (erodata as usize).into(),
               MapType::Identical,
               MapPermission::R
       ), None);

       println!("mapping .data section");
       memory_set.push(MapArea::new(
               (sdata as usize).into(),
               (edata as usize).into(),
               MapType::Identical,
               MapPermission::R | MapPermission::W
       ), None);

       println!("mapping .bss section");
       memory_set.push(MapArea::new(
               (sbss_with_stack as usize).into(),
               (ebss as usize).into(),
               MapType::Identical,
               MapPermission::R | MapPermission::W
       ), None);

       // map available physical frames  
       println!("mapping available physical frames");
       memory_set.push(MapArea::new(
               (ekernel as usize).into(),
               (MEMORY_END as usize).into(),
               MapType::Identical,
               MapPermission::R | MapPermission::W
       ), None);

       // map MMIO registers
       //TODO
       
       memory_set
    }
    
    /// create application address space
    /// Include sections in elf, trampoline, trap_context
    /// also returns user_sp_base and entry point of each app
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();
        
        /* 
            map trampoline: 
            trampoline of application and kernel mapped onto the same physical frame
        */
        memory_set.map_trampoline();
        
        // map program headers of elf, with U flags 
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(
                    start_va,
                    end_va,
                    MapType::Framed,
                    map_perm
                );
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])
                );
            }
        }
        
        // map user stack with U flags and guard page    
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        let guard_page_size = PAGE_SIZE;
        user_stack_bottom += guard_page_size;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U
        ), None);

        // map TrapContext 
        memory_set.push(MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W
        ), None);
        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }

    /*
        We must ensure that the transition involving the `satp` CSR switch is a smooth one. 
        By "smooth," we mean that the instructions for switching `satp` and the subsequent 
        instruction are located at adjacent virtual addresses. While the physical addresses 
        of these two instructions are generally adjacent as well, the address translation 
        process they go through is different—changing `satp` leads to a lookup in a different 
        multi-level page table. This requires that the mappings in the vicinity of the `satp` 
        switch instructions, both before and after, have a certain sense of continuity.

        Fortunately, we have achieved this. The instruction that writes to `satp` and the 
        subsequent instruction are both located within the code segment of the kernel's 
        memory layout. After the switch, these instructions constitute an identity mapping, 
        and before the switch, they are considered to be fetching directly from physical 
        addresses. This aligns perfectly with our expectations: even after switching the 
        address space, instructions should be able to execute in a continuous manner.
    */
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
    
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    /// clone a same `MemorySet` => sys_fork
    pub fn from_existed_user(user_space: &MemorySet) -> MemorySet {
        let mut memory_set = Self::new_bare();
        
        /* 
            map trampoline:
            because the trampoline segment is not included in MemorySet::areas.
        */ 
        memory_set.map_trampoline();

        // copy program_sections/trap_context/user_stack
        for area in user_space.areas.iter() {
            let new_area = MapArea::from_another(area);
            memory_set.push(new_area, None);
            // copy data from another user_space
            for vpn in area.vpn_range {
                let src_ppn = user_space.translate(vpn).unwrap().ppn();
                let dst_ppn = memory_set.translate(vpn).unwrap().ppn();
                dst_ppn.get_bytes_array().copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        memory_set   
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) >> 1).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) >> 1).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) >> 1).into();
    
    println!("detect mid_text access permission...");
    assert!(
        !kernel_space
        .translate(mid_text.floor())
        .unwrap()
        .writeable()
    );

    println!("detect mid_rodata access permission...");
    assert!(
        !kernel_space
        .translate(mid_rodata.floor())
        .unwrap()
        .writeable()
    );

    println!("detect mid_data access permission...");
    assert!(
        !kernel_space
        .translate(mid_data.floor())
        .unwrap()
        .executable()
    );
    println!("remap_test passed");
}
