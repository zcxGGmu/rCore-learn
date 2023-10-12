/*!
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
use super::{PTEFlags, PageTable};
use super::{PhysPageNum, VirtAddr, VirtPageNum};
use super::{StepByOne, VPNRange};
use crate::config::PAGE_SIZE;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use bitflags::*;

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
            vpn_range: VPNRange::new(another.vpn_range.get_start(),
                                    another.vpn_range.get_end()),
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
        permission: MapPermission
    ) {
       self.push(
           MapArea::new(start_va, end_va, MapType::Framed, permission),
           None
       ); 
    }
}
