#![no_std]

extern crate alloc;

///
/// Simple chopin memory allocator
/// just does heap scans
///
/// Every allocation has a leading u32
/// representing its length
///
pub struct EarlyKernelAllocator {
    heap_start: usize,
    heap_end: usize,
}

impl EarlyKernelAllocator {
    pub unsafe fn new(start: usize, end: usize) -> EarlyKernelAllocator {
        // println("Alloc begins at:");
        // print_u64(start as u64);
        for i in start..end {
            let ptr = i as *mut u8;
            ptr.write(0x00);
        }
        EarlyKernelAllocator {
            heap_start: start,
            heap_end: end,
        }
    }
}

unsafe impl alloc::alloc::GlobalAlloc for EarlyKernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        // Find a null block to allocate
        // block must be at least layout.size + 4

        let min_free_size = layout.size() + 4;
        let min_alignment = layout.align();

        let mut alignment_zero_mask = min_alignment >> 1;

        loop{
            let old_mask = alignment_zero_mask;
            alignment_zero_mask = alignment_zero_mask | (alignment_zero_mask >> 1);

            if old_mask == alignment_zero_mask{
                break;
            }
        }
        
        let mut current_cell = self.heap_start as *mut u8;

        loop {
            if current_cell >= self.heap_end as *mut u8 {
                return core::ptr::null_mut();
            } else if *current_cell == 0 {
                // Check if this block has enough bytes
                
                let current_cell_addr = current_cell as usize;

                if current_cell_addr & alignment_zero_mask != 0{
                    current_cell = current_cell.wrapping_add(1);
                    continue;
                }

                if current_cell_addr & min_alignment != min_alignment{
                    current_cell = current_cell.wrapping_add(1);
                    continue;
                }

                // if the current cell doesnt fit alignment 
                // then we skip this byte 

                let cell_start = current_cell;
                loop {
                    current_cell = current_cell.add(1);

                    if current_cell >= self.heap_end as *mut u8 || *current_cell != 0 {
                        break;
                    }
                }

                let byte_count = current_cell.offset_from(cell_start) as usize;

                if byte_count >= min_free_size {
                    // Create the allocation

                    let len_bytes = (layout.size() as u32).to_ne_bytes();

                    cell_start.add(0).write(len_bytes[0]);
                    cell_start.add(1).write(len_bytes[1]);
                    cell_start.add(2).write(len_bytes[2]);
                    cell_start.add(3).write(len_bytes[3]);

                    let alloc_ptr = cell_start.add(4);
                    return alloc_ptr;
                }
            } else {
                // assuming this is an allocation, read the length
                // and jump to after it
                let alloc_len = u32::from_ne_bytes([
                    current_cell.add(0).read(),
                    current_cell.add(1).read(),
                    current_cell.add(2).read(),
                    current_cell.add(3).read(),
                ]);

                current_cell = current_cell.add(alloc_len as usize + 4);
            }
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        let alloc_len = u32::from_ne_bytes([*ptr.sub(4), *ptr.sub(3), *ptr.sub(2), *ptr.sub(1)]);

        // Zero the allocation
        for i in 0..alloc_len {
            ptr.add(i as usize).write(0x00);
        }

        // Zero the metadata
        ptr.sub(1).write(0x00);
        ptr.sub(2).write(0x00);
        ptr.sub(3).write(0x00);
        ptr.sub(4).write(0x00);
    }
}

pub enum AllocatorVariant {
    None,
    Early(EarlyKernelAllocator),
}

pub struct KernelAllocator {
    pub allocator: AllocatorVariant,
}

unsafe impl alloc::alloc::GlobalAlloc for KernelAllocator {
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        match &self.allocator {
            AllocatorVariant::None => panic!("Attempt to allocate with uninitialized allocator"),
            AllocatorVariant::Early(ek) => ek.dealloc(ptr, layout),
        }
    }

    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        match &self.allocator {
            AllocatorVariant::None => panic!("Attempt to allocate with uninitialized allocator"),
            AllocatorVariant::Early(ek) => ek.alloc(layout),
        }
    }
}

#[global_allocator]
pub static mut ALLOCATOR: KernelAllocator = KernelAllocator {
    allocator: AllocatorVariant::None,
};
