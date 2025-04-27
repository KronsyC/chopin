mod bootstrap;
pub mod virt_map;

pub use bootstrap::bootstrap_pt;

use crate::frame_table::{FrameTable, MemoryAllocation};

unsafe fn pte_pointer_as_mut_slice(pte_ptr: *mut PageTableEntry) -> &'static mut [PageTableEntry] {
    // assume 512 entries
    unsafe { core::slice::from_raw_parts_mut(pte_ptr, 512) }
}

pub struct IntermediaryPageTableResult {
    pub page_table: PageTable,
    pub physical_starting_memory_address: usize,
    pub allocated: bool,
}

pub struct PageTable {
    pub entries: &'static mut [PageTableEntry],
}

impl PageTable {
    pub unsafe fn is_free(
        &self,
        l1_index: PageTableIndex,
        l2_index: PageTableIndex,
        l3_index: PageTableIndex,
    ) -> bool {
        let e1 = &self.entries[l1_index.as_addr()];

        if e1.is_unused() {
            true
        } else {
            let p2 = unsafe { Self::from_pointer(e1.phys_addr()) };

            let e2 = &p2.entries[l2_index.as_addr()];

            if e2.is_unused() {
                true
            } else {
                let p3 = unsafe { Self::from_pointer(e2.phys_addr()) };
                let e3 = &p3.entries[l3_index.as_addr()];

                if e3.is_unused() { true } else { false }
            }
        }
    }

    ///
    /// Make an actual virtual memory mapping
    ///
    pub unsafe fn make_mapping(&mut self, index: PageTableIndex, address: usize, flags: u64) {
        let e = &mut self.entries[index.as_addr()];

        // TODO: Make sure e is unallocated

        e.set(address as u64, flags);
    }
    pub unsafe fn from_pointer(pointer: usize) -> Self {
        // assume 512 entries
        let elements =
            unsafe { core::slice::from_raw_parts_mut(pointer as *mut PageTableEntry, 512) };

        Self { entries: elements }
    }

    pub unsafe fn allocate_intermediary(
        &mut self,
        index: PageTableIndex,
        frame_table: &mut FrameTable,
    ) -> Result<IntermediaryPageTableResult, ()> {
        let e = &mut self.entries[index.as_addr()];

        if e.is_unused() {
            // Good to allocate
            let mem = frame_table
                .alloc_front(1, crate::frame_table::FrameState::PageTable, 0)
                .unwrap();

            let pt = unsafe { PageTable::from_pointer(mem.phys_addr) };

            e.set(mem.phys_addr as u64, PageTableEntry::FLAG_V);

            Ok(IntermediaryPageTableResult {
                page_table: pt,
                physical_starting_memory_address: mem.phys_addr,
                allocated: true,
            })
        } else if e.phys_addr() != 0 && e.is_valid() && e.kind() == PTEKind::NextLevel {
            log::debug!("Using pre-existing allocation for allocate_intermediary");

            Ok(IntermediaryPageTableResult {
                page_table: unsafe { PageTable::from_pointer(e.phys_addr()) },
                physical_starting_memory_address: e.phys_addr(),
                allocated: false,
            })
        } else {
            log::warn!("Failed to allocate intermediary as it is already allocated for mapping");
            Err(())
        }
    }

    ///
    /// This will allocate a page to be used as a page table
    /// and also map this sub-table into the page table
    /// at the self-ref address
    ///
    /// Returns the physical address of the page
    ///
    pub unsafe fn meta_allocate_page_table(
        &mut self,
        frame_table: &mut FrameTable,
    ) -> MemoryAllocation {
        let self_ref_root_index = PageTableIndex::new(280);

        let allocation = frame_table
            .alloc_front(1, crate::frame_table::FrameState::PageTable, 0)
            .unwrap();

        unsafe { allocation.zero() };

        let offset_from_base = allocation.phys_addr - frame_table.root_frame_address();
        let pages_offset_from_base = offset_from_base >> 12;

        // log::debug!(
        //     "Meta-allocating new page on page table self-ref section at index #{pages_offset_from_base}"
        // );

        let l1_index = (pages_offset_from_base >> 9) & 0x1FF;
        let l2_index = (pages_offset_from_base) & 0x1FF;

        // log::debug!("Index at 280 > {l1_index} > {l2_index}");

        let l1_pt = unsafe { PageTable::from_pointer(self.entries[self_ref_root_index.as_addr()].phys_addr()) };
            // unsafe { self.allocate_intermediary(self_ref_root_index, frame_table) }.unwrap();

        // assert!(
        //     !l2.allocated,
        //     "L2 SHOULD HAVE BEEN ALLOCATED BY BOOTSTRAP - SOMETHING IS WRONG"
        // );

        let l2_ref = &mut l1_pt.entries[l1_index];

        if l2_ref.is_unused() {
            // We are allocating new intermediary table
            let intermediary = frame_table
                .alloc_front(1, crate::frame_table::FrameState::PageTable, 0)
                .unwrap();

            unsafe { intermediary.zero() };

            l2_ref.set(intermediary.phys_addr as u64, PageTableEntry::FLAG_V);

            let intermediary_pt = unsafe { PageTable::from_pointer(intermediary.phys_addr) };

            assert_eq!(
                l2_index, 0,
                "L3 index should be 0, otherwise a core assumption is incorrect"
            );

            // Intermediary should be exactly 4096 bytes after allocation
            assert_eq!(
                allocation.phys_addr + 4096,
                intermediary.phys_addr,
                "Intermediary should come immediately after leaf allocation"
            );

            intermediary_pt.entries[0].set(
                allocation.phys_addr as u64,
                PageTableEntry::FLAG_V | PageTableEntry::FLAG_R | PageTableEntry::FLAG_W,
            );

            intermediary_pt.entries[1].set(intermediary.phys_addr as u64, PageTableEntry::FLAG_V);

            // log::debug!("Successfully self-mapped page table entry with new-mapped l2");
            // log::debug!("Intermediary created at memaddr: {:#X?}", intermediary.phys_addr);

            allocation
        } else {
            // Best case - allocate within existing l2
            let l2_pt = unsafe { PageTable::from_pointer(l2_ref.phys_addr())};


            let entry = &mut l2_pt.entries[l2_index];

            
            assert!(entry.is_unused(), "Target PTE is unused");

            entry.set(
                allocation.phys_addr as u64,
                PageTableEntry::FLAG_V | PageTableEntry::FLAG_R | PageTableEntry::FLAG_W,
            );

            // log::debug!("Successfully self-mapped page table entry");
            allocation
        }
    }

    ///
    /// Create a new allocation within the page table
    ///
    ///
    pub fn create_allocation_pages(
        &mut self,
        page_count: usize,
        frame_table: &mut FrameTable,
        flags: u64,
    ) -> Result<usize, ()> {
        assert!(
            page_count <= 512,
            "Current allocator can only allocate up to 512 pages at a time (l1 max)"
        );

        if page_count > 512 {
            panic!()
        }

        // TODO: Enable cross-level allocations
        //
        // For now, we just allocate consecutive blocks
        // locally to an l1 table
        //
        // We should also potentially randomize allocations a bit
        //
        // and prioritize allocating within existing tables over making new tables when possible
        //
        // theres lots of parameters here that must be investigated
        //
        // for now, we just linearly allocate
        //
        // so the max allocation size is
        // 4KiB * 512 = 2MiB
        //
        // TODO: Also emit failure reasons

        // L3 Loop
        for (l3_index, e) in self.entries.iter_mut().enumerate() {
            match e.next_level(frame_table) {
                Some(l2_table) => {
                    // L2 Loop

                    for (l2_index, e) in l2_table.entries.iter_mut().enumerate() {
                        match e.next_level(frame_table) {
                            Some(l1_table) => {
                                // find page_count consecutive free entries
                                let earliest_free_segment =
                                    l1_table.first_free_cells_accommodating(page_count);

                                if let Some(l1_index) = earliest_free_segment {
                                    let pages_to_allocate =
                                        &mut l1_table.entries[l1_index..l1_index + page_count];

                                    let frames = frame_table.alloc_back(page_count, crate::frame_table::FrameState::Kernel, 0).expect("Failed to allocate backing page memory from frame table");

                                    assert_eq!(
                                        frames.page_count,
                                        pages_to_allocate.len(),
                                        "PTE count and frame allocation length dont match"
                                    );
                                    for (index, page) in pages_to_allocate.iter_mut().enumerate() {
                                        let physical_page_address =
                                            frames.phys_addr + crate::PAGE_SIZE_B * index;
                                        page.set(physical_page_address as u64, flags);
                                    }

                                    // We now have a full allocation, just determine the virtual
                                    // address now
                                    //
                                    // SIGN | VPN[2] | VPN[1] | VPN[0] | OFFSET
                                    //  25  |   9    |   9    |   9    |  12
                                    //

                                    let vpn_index = l1_index | (l2_index << 9) | (l3_index << 18);

                                    // Zero offset
                                    let vpn_index = vpn_index << 12;

                                    // Sign extension logic
                                    const S_EXT : usize = 0b1111111111111111111111111000000000000000000000000000000000000000;
                                    const EXT: [usize; 2] = [0, S_EXT];

                                    let extend = EXT[l3_index >> 8];

                                    let vpn_index = vpn_index | extend;

                                    return Ok(vpn_index);
                                } else {
                                    continue;
                                }
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                }
                None => {
                    // Scan next
                    continue;
                }
            }
        }

        Err(())
    }

    ///
    /// Yield the first index of a run capable of storing `page_count`
    ///
    pub fn first_free_cells_accommodating(&self, page_count: usize) -> Option<usize> {
        let mut current_run_length = 0usize;

        for (i, page) in self.entries.iter().enumerate() {
            if matches!(page.kind(), PTEKind::NextLevel) {
                current_run_length += 1;

                if current_run_length == page_count {
                    return Some(i - current_run_length);
                }
            } else {
                current_run_length = 0;
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PTEKind {
    NextLevel,
    ReadOnly,
    ReadWrite,
    ExecOnly,
    ReadExecute,
    ReadWriteExecute,

    ///
    /// Reserved #1
    ///
    /// this is the case of write-only
    ///
    _Reserved1,

    ///
    /// Reserved #2
    ///
    /// this is the case of write-execute
    ///
    _Reserved2,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    ///
    /// Is this table entry valid
    ///
    pub const FLAG_V: u64 = 1 << 0;

    ///
    /// Should this memory be readable
    ///
    pub const FLAG_R: u64 = 1 << 1;

    ///
    /// Should this memory be writable
    ///
    pub const FLAG_W: u64 = 1 << 2;

    ///
    /// Should this memory be executable
    ///
    pub const FLAG_X: u64 = 1 << 3;

    ///
    /// Is this memory accessible from user mode
    ///
    pub const FLAG_U: u64 = 1 << 4;

    ///
    /// Is this memory globally available
    ///
    /// does some funky optimizations during process switching
    ///
    pub const FLAG_G: u64 = 1 << 5;

    ///
    /// Is this memory currently being accessed
    ///
    /// this is only really used by the hardware
    ///
    pub const FLAG_A: u64 = 1 << 6;

    ///
    /// Is this memory dirty
    ///
    /// this is set by the hardware after a write
    ///
    pub const FLAG_D: u64 = 1 << 7;

    pub fn is_valid(&self) -> bool {
        self.0 & Self::FLAG_V != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.0 & (Self::FLAG_R | Self::FLAG_W | Self::FLAG_X) != 0
    }

    pub fn phys_addr(&self) -> usize {
        let ppn = (self.0 >> 10) & 0xFFFFFFFFFFF;
        (ppn << 12) as usize
    }

    pub fn set(&mut self, phys_addr: u64, flags: u64) {
        let ppn = phys_addr >> 12;
        self.0 = (ppn << 10) | flags;
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    pub fn kind(&self) -> PTEKind {
        let x = ((self.0 >> 3) & 1) != 0;
        let w = ((self.0 >> 2) & 1) != 0;
        let r = ((self.0 >> 1) & 1) != 0;

        match (x, w, r) {
            (false, false, false) => PTEKind::NextLevel,
            (true, false, false) => PTEKind::ExecOnly,
            (true, true, false) => PTEKind::_Reserved2,
            (true, true, true) => PTEKind::ReadWriteExecute,
            (false, true, true) => PTEKind::ReadWrite,
            (false, false, true) => PTEKind::ReadOnly,
            (false, true, false) => PTEKind::_Reserved1,
            (true, false, true) => PTEKind::ReadExecute,
        }
    }

    pub fn set_kind(&mut self, kind: PTEKind) {
        // Clear bits R, W, X (bits 1, 2, 3)
        self.0 &= !(0b111 << 1);

        // Set based on kind
        let (x, w, r) = match kind {
            PTEKind::NextLevel => (false, false, false),
            PTEKind::ExecOnly => (true, false, false),
            PTEKind::ReadOnly => (false, false, true),
            PTEKind::ReadExecute => (true, false, true),
            PTEKind::ReadWrite => (false, true, true),
            PTEKind::ReadWriteExecute => (true, true, true),
            PTEKind::_Reserved1 => (false, true, false),
            PTEKind::_Reserved2 => (true, true, false),
        };

        self.0 |= ((r as u64) << 1) | ((w as u64) << 2) | ((x as u64) << 3);
    }

    pub fn next_level(
        &self,
        frame_table: &mut crate::frame_table::FrameTable,
    ) -> Option<PageTable> {
        let is_nextlevel = matches!(self.kind(), PTEKind::NextLevel);
        let has_value = self.phys_addr() != 0;

        match (is_nextlevel, has_value) {
            (true, true) => {
                // Next Level
                let pt_address = self.phys_addr();

                // This is a pointer to the first entry in a PT

                let pt = unsafe { PageTable::from_pointer(pt_address) };

                Some(pt)
            }
            (true, false) => {
                // Allocate next level and return it

                let next_level =
                    frame_table.alloc_front(1, crate::frame_table::FrameState::PageTable, 0);

                match next_level {
                    Some(allocation) => {
                        let pt = unsafe { PageTable::from_pointer(allocation.phys_addr) };
                        Some(pt)
                    }
                    None => {
                        // Cascade the failure
                        None
                    }
                }
            }
            (false, true) => {
                // This is a non-standard page, dont touch
                None
            }
            (false, false) => {
                // This is a non-standard empty page
                // dont touch
                None
            }
        }
    }
}

///
/// A page table index
///
/// must not exceed 512
///
#[derive(Clone, Copy, Debug)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    pub fn new(val: u16) -> Self {
        assert!(val <= 512, "Value within bounds");
        Self(val)
    }

    pub fn as_addr(&self) -> usize {
        self.0 as usize
    }
}

impl From<PageTableIndex> for usize {
    fn from(value: PageTableIndex) -> Self {
        value.0 as usize
    }
}
