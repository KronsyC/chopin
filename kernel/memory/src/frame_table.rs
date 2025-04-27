use alloc::vec::Vec;
use core::ops::Add;

///
/// This data structure tracks all frames
/// available on the system
///
///
pub struct FrameTable {
    pub segments: Vec<FrameSegment>,
}

///
/// This structure tracks all frame segments
///
/// we use this approach because system memory
///
pub struct FrameSegment {
    ///
    /// The address at which the page metadata
    /// tracking starts at
    ///
    /// Each metadata entry is 2 bytes
    ///
    pub frame_metadata_start_addr: usize,

    ///
    /// The address of the first page
    ///
    pub first_page_addr: usize,

    ///
    /// The number of pages
    ///
    pub page_count: usize,
}

impl FrameTable {
    /// Tries to allocate `count` contiguous pages from any segment.
    pub fn alloc_front(
        &mut self,
        count: usize,
        state: FrameState,
        pid: u16,
    ) -> Option<MemoryAllocation> {
        for segment in &mut self.segments {
            if let Some(alloc) = segment.alloc_front(count, state, pid) {
                return Some(alloc);
            }
        }

        None
    }

    /// Tries to allocate `count` contiguous pages from the back of any segment.
    pub fn alloc_back(
        &mut self,
        count: usize,
        state: FrameState,
        pid: u16,
    ) -> Option<MemoryAllocation> {
        for segment in &mut self.segments {
            if let Some(alloc) = segment.alloc_back(count, state, pid) {
                return Some(alloc);
            }
        }

        None
    }
}
impl FrameSegment {
    pub unsafe fn get_page(&self, idx: usize) -> &'static mut [u8; 4096] {
        assert!(idx < self.page_count, "get_page: index out of bounds");
        unsafe { &mut *(self.first_page_addr.add(idx * 4096) as *mut [u8; 4096]) }
    }

    pub unsafe fn get_metadata(&self, idx: usize) -> &'static mut FrameMetadataEntry {
        assert!(idx < self.page_count, "get_metadata: index out of bounds");
        unsafe {
            &mut *(self
                .frame_metadata_start_addr
                .add(idx * core::mem::size_of::<FrameMetadataEntry>())
                as *mut FrameMetadataEntry)
        }
    }

    ///
    /// Initialize a frame segment from a continuous memory region.
    /// Returns `None` if the region is too small to contain both metadata and at least one page.
    ///
    pub fn initialize(start_address: usize, size: usize) -> Option<FrameSegment> {
        const PAGE_SIZE: usize = 4096;
        const METADATA_SIZE_PER_PAGE: usize = core::mem::size_of::<FrameMetadataEntry>();

        // Max number of pages if metadata were "free"
        let max_possible_pages = size / (PAGE_SIZE + METADATA_SIZE_PER_PAGE);

        if max_possible_pages == 0 {
            return None;
        }

        // Compute actual metadata size and align it up to the next page
        let raw_metadata_bytes = max_possible_pages * METADATA_SIZE_PER_PAGE;
        let aligned_metadata_bytes = align_up(raw_metadata_bytes, PAGE_SIZE);

        // Now compute how many pages we can still fit after reserving metadata
        let usable_bytes = size - aligned_metadata_bytes;
        let usable_page_count = usable_bytes / PAGE_SIZE;

        if usable_page_count == 0 {
            return None;
        }

        Some(FrameSegment {
            frame_metadata_start_addr: start_address,
            first_page_addr: start_address + aligned_metadata_bytes,
            page_count: usable_page_count,
        })
    }

    /// Allocates `count` contiguous 4K pages from this segment.
    /// Returns a `MemoryAllocation` with metadata set, or `None` if no space.
    pub fn alloc_front(
        &mut self,
        count: usize,
        state: FrameState,
        pid: u16,
    ) -> Option<MemoryAllocation> {
        let end_idx = self.page_count.saturating_sub(count);

        for idx in 0..=end_idx {
            // Check if the range [idx, idx + count) is all free
            let mut all_free = true;
            for offset in 0..count {
                let meta = unsafe { self.get_metadata(idx + offset) };
                if meta.state() != FrameState::Free {
                    all_free = false;
                    break;
                }
            }

            if all_free {
                for offset in 0..count {
                    let meta = unsafe { self.get_metadata(idx + offset) };
                    meta.set_state(state);
                    meta.set_pid(pid);
                }

                return Some(MemoryAllocation {
                    phys_addr: self.first_page_addr + idx * 4096,
                    page_count: count,
                    pid,
                    state,
                });
            }
        }

        None
    }

    /// Allocates `count` contiguous 4K pages from the end (high address side) of this segment.
    /// Returns a `MemoryAllocation` with metadata set, or `None` if no space.
    pub fn alloc_back(
        &mut self,
        count: usize,
        state: FrameState,
        pid: u16,
    ) -> Option<MemoryAllocation> {
        if count == 0 || count > self.page_count {
            return None;
        }

        // Start from the end and look backwards for a free run
        let mut idx = self.page_count;

        while idx >= count {
            let start = idx - count;
            let mut all_free = true;

            for offset in 0..count {
                let meta = unsafe { self.get_metadata(start + offset) };
                if meta.state() != FrameState::Free {
                    all_free = false;
                    break;
                }
            }

            if all_free {
                for offset in 0..count {
                    let meta = unsafe { self.get_metadata(start + offset) };
                    meta.set_state(state);
                    meta.set_pid(pid);
                }

                let phys_addr = self.first_page_addr + start * 4096;

                return Some(MemoryAllocation {
                    phys_addr,
                    page_count: count,
                    pid,
                    state,
                });
            }

            idx -= 1;
        }

        None
    }
}

/// Represents the current usage of a physical page frame.
///
/// Each frame in physical memory is assigned a `FrameState`,
/// allowing the kernel to track ownership, purpose, and reclaimability.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameState {
    /// The frame is completely unused and available for allocation.
    ///
    /// When searching for free memory, only pages in this state
    /// are considered candidates. Upon allocation, this will be updated.
    Free = 0,

    /// The frame is allocated, but its specific purpose is not tracked further.
    ///
    /// This can be used as a general-purpose "in use" state, often for simple
    /// memory allocators or in situations where fine-grained tracking isn't needed.
    ///
    /// More specific states (like Kernel, PageTable, etc.) are preferred if you need
    /// tighter accounting or cleanup support.
    Used = 1,

    /// The frame is owned by the kernel itself.
    ///
    /// This might include:
    /// - Kernel heap allocations
    /// - Static memory pools
    /// - Kernel stacks
    ///
    /// These are never directly accessible by user processes, and they should not
    /// be freed as part of process cleanup.
    Kernel = 2,

    /// The frame is owned by a user-space process.
    ///
    /// This applies to:
    /// - User stacks
    /// - User heap pages
    /// - mmap'd anonymous memory
    ///
    /// These pages should be freed when the owning process exits.
    /// The owning PID should be recorded for cleanup purposes.
    User = 3,

    /// The frame holds a **page table** at any level (L0, L1, L2).
    ///
    /// These pages are crucial for virtual memory translation and must be managed
    /// by the kernel. They are typically kernel-owned, but tied to a process.
    ///
    /// They must be:
    /// - Allocated via kernel-only logic
    /// - Zeroed before use
    /// - Freed carefully on process destruction
    PageTable = 4,

    /// The frame is reserved and cannot be allocated.
    ///
    /// This state is used to mark memory that:
    /// - Is reserved by the firmware or device tree
    /// - Holds critical kernel or device buffers
    /// - Is outside your managed usable regions
    ///
    /// These frames must never be reused, and will not appear in allocator searches.
    Reserved = 5,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameMetadataEntry {
    raw: u32,
}

impl FrameMetadataEntry {
    pub fn new() -> Self {
        Self { raw: 0 }
    }

    pub fn state(&self) -> FrameState {
        match self.raw & 0xF {
            0 => FrameState::Free,
            1 => FrameState::Used,
            2 => FrameState::Kernel,
            3 => FrameState::User,
            4 => FrameState::PageTable,
            5 => FrameState::Reserved,
            _ => FrameState::Used, // fallback
        }
    }

    pub fn set_state(&mut self, state: FrameState) {
        self.raw = (self.raw & !0xF) | (state as u32);
    }

    pub fn flags(&self) -> u8 {
        ((self.raw >> 4) & 0xF) as u8
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.raw = (self.raw & !(0xF << 4)) | (((flags as u32) & 0xF) << 4);
    }

    pub fn pid(&self) -> u16 {
        ((self.raw >> 8) & 0xFFFF) as u16
    }

    pub fn set_pid(&mut self, pid: u16) {
        self.raw = (self.raw & !(0xFFFF << 8)) | ((pid as u32) << 8);
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// Represents a successful allocation of physical memory.
#[derive(Clone)]
pub struct MemoryAllocation {
    /// The physical address of the first page in this allocation.
    pub phys_addr: usize,

    /// The number of 4K pages allocated.
    pub page_count: usize,

    /// The owning process ID (0 = kernel).
    pub pid: u16,

    /// The type of allocation (e.g., User, Kernel, PageTable).
    pub state: FrameState,
}

impl MemoryAllocation {
    /// Zeroes all pages in this allocation.
    ///
    /// # Safety
    /// This function assumes that all pages are physically mapped
    /// and writable by the kernel.
    pub unsafe fn zero(&self) {
        for i in 0..self.page_count {
            let page_ptr = (self.phys_addr + i * 4096) as *mut u8;
            unsafe { core::ptr::write_bytes(page_ptr, 0, 4096) };
        }
    }

    /// Returns an iterator over each 4K page's physical address.
    pub fn iter_pages(&self) -> impl Iterator<Item = usize> + use<'_> {
        (0..self.page_count).map(move |i| self.phys_addr + i * 4096)
    }

    /// # Safety
    ///
    /// - `addr` must be valid and mapped in the current address space
    /// - Memory must be initialized to `T`-compatible values
    /// - Caller must ensure correct lifetime management
    pub unsafe fn as_slice<T>(&self) -> &'static mut [T] {
        let ptr = self.phys_addr as *mut T;
        let count = self.page_count * crate::PAGE_SIZE_B / core::mem::size_of::<T>();
        unsafe { core::slice::from_raw_parts_mut(ptr, count) }
    }
}

impl core::fmt::Debug for MemoryAllocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let size_kib = self.page_count * 4; // 4 KiB per page
        write!(
            f,
            "MemoryAllocation {{ addr: {:#x}, pages: {}, size: {} KiB, pid: {}, state: {:?} }}",
            self.phys_addr, self.page_count, size_kib, self.pid, self.state,
        )
    }
}
