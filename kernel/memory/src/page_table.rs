use crate::frame_table::{FrameTable, MemoryAllocation};

unsafe fn pte_pointer_as_mut_slice(pte_ptr: *mut PageTableEntry) -> &'static mut [PageTableEntry] {
    // assume 512 entries
    unsafe { core::slice::from_raw_parts_mut(pte_ptr, 512) }
}

pub struct PageTable {
    pub entries: &'static mut [PageTableEntry],
}

impl PageTable {
    pub unsafe fn from_pointer(pointer: usize) -> Self {
        // assume 512 entries
        let elements =
            unsafe { core::slice::from_raw_parts_mut(pointer as *mut PageTableEntry, 512) };

        Self { entries: elements }
    }

    ///
    /// Create a new allocation within the page table
    ///
    ///
    pub fn create_allocation_pages(&mut self, page_count: usize, frame_table: &mut FrameTable, flags : u64) {
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

                    for (l2_index, e) in l2_table.entries.iter_mut().enumerate(){
                        match e.next_level(frame_table){
                            Some(l1_table) => {
                                // find page_count consecutive free entries 
                                let earliest_free_segment = l1_table.first_free_cells_accommodating(page_count);

                                if let Some(l1_index) = earliest_free_segment{

                                    let pages_to_allocate = &mut l1_table.entries[l1_index..l1_index+page_count];


                                    let frames = frame_table.alloc_back(page_count, crate::frame_table::FrameState::Kernel, 0).expect("Failed to allocate backing page memory from frame table");

                                    assert_eq!(frames.page_count, pages_to_allocate.len(), "PTE count and frame allocation length dont match");
                                    for (index, page) in pages_to_allocate.iter_mut().enumerate(){

                                        let physical_page_address = frames.phys_addr + crate::PAGE_SIZE_B * index;
                                        page.set(physical_page_address as u64, flags);
                                    }

                                    // We now have a full allocation, just determine the virtual
                                    // address now
                                    //
                                    // SIGN | VPN[2] | VPN[1] | VPN[0] | OFFSET
                                    //  25  |   9    |   9    |   9    |  12
                                    //


                                    let vpn_index = l1_index 
                                        | (l2_index << 9)
                                        | (l3_index << 18);

                                    // Zero offset
                                    let vpn_index = vpn_index << 12;

                                    
                                }
                                else{
                                    continue;
                                }
                            },
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
    }


    ///
    /// Yield the first index of a run capable of storing `page_count`
    ///
    pub fn first_free_cells_accommodating(&self, page_count : usize) -> Option<usize>{
        let mut current_run_length = 0usize;

        for (i, page) in self.entries.iter().enumerate(){
            if matches!(page.kind(), PTEKind::NextLevel){
                current_run_length += 1;

                if current_run_length == page_count{
                    return Some(i - current_run_length)
                }
            }
            else{
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

                let next_level = frame_table.alloc_front(1, crate::frame_table::FrameState::PageTable, 0);

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
