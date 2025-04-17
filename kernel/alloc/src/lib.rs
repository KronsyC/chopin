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

// unsafe impl alloc::alloc::GlobalAlloc for EarlyKernelAllocator {
//     unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
//         // Find a null block to allocate
//         // block must be at least layout.size + 4
//
//         // layout.align_to
//         let min_free_size = layout.size() + 4;
//         let min_alignment = layout.align();
//
//         let alignment_zero_mask = min_alignment - 1;
//
//         
//         let mut current_cell = self.heap_start as *mut u8;
//
//         loop {
//             if current_cell >= self.heap_end as *mut u8 {
//                 return core::ptr::null_mut();
//             } else if *current_cell == 0 {
//                 // Check if this block has enough bytes
//                 
//                 let current_cell_addr = current_cell as usize;
//
//                 if current_cell_addr & alignment_zero_mask != 0{
//                     current_cell = current_cell.wrapping_add(1);
//                     continue;
//                 }
//
//                 if current_cell_addr & min_alignment != min_alignment{
//                     current_cell = current_cell.wrapping_add(1);
//                     continue;
//                 }
//
//                 // if the current cell doesnt fit alignment 
//                 // then we skip this byte 
//
//                 let cell_start = current_cell;
//                 loop {
//                     current_cell = current_cell.add(1);
//
//                     if current_cell >= self.heap_end as *mut u8 || *current_cell != 0 {
//                         break;
//                     }
//                 }
//
//                 let byte_count = current_cell.offset_from(cell_start) as usize;
//
//                 if byte_count >= min_free_size {
//                     // Create the allocation
//
//                     let len_bytes = (layout.size() as u32).to_ne_bytes();
//
//                     cell_start.add(0).write(len_bytes[0]);
//                     cell_start.add(1).write(len_bytes[1]);
//                     cell_start.add(2).write(len_bytes[2]);
//                     cell_start.add(3).write(len_bytes[3]);
//
//                     let alloc_ptr = cell_start.add(4);
//                     return alloc_ptr;
//                 }
//             } else {
//                 // assuming this is an allocation, read the length
//                 // and jump to after it
//                 let alloc_len = u32::from_ne_bytes([
//                     current_cell.add(0).read(),
//                     current_cell.add(1).read(),
//                     current_cell.add(2).read(),
//                     current_cell.add(3).read(),
//                 ]);
//
//                 current_cell = current_cell.add(alloc_len as usize + 4);
//             }
//         }
//     }
//     unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
//         let alloc_len = u32::from_ne_bytes([*ptr.sub(4), *ptr.sub(3), *ptr.sub(2), *ptr.sub(1)]);
//
//         // Zero the allocation
//         for i in 0..alloc_len {
//             ptr.add(i as usize).write(0x00);
//         }
//
//         // Zero the metadata
//         ptr.sub(1).write(0x00);
//         ptr.sub(2).write(0x00);
//         ptr.sub(3).write(0x00);
//         ptr.sub(4).write(0x00);
//     }
// }
unsafe impl alloc::alloc::GlobalAlloc for EarlyKernelAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let min_free_size = layout.size() + 4;
        let required_alignment = layout.align();
        
        // Get aligned start address for our metadata
        let mut current_cell = self.heap_start as *mut u8;
        
        while current_cell < (self.heap_end as *mut u8) {
            // First align the metadata block itself
            let metadata_addr = current_cell as usize;
            let metadata_alignment = core::mem::align_of::<u32>();
            let aligned_metadata = (metadata_addr + metadata_alignment - 1) & !(metadata_alignment - 1);
            current_cell = aligned_metadata as *mut u8;
            
            // Now calculate where the actual allocation would start
            let potential_alloc = current_cell.add(4);
            // Align the allocation address to the required alignment
            let alloc_addr = potential_alloc as usize;
            let aligned_alloc = (alloc_addr + required_alignment - 1) & !(required_alignment - 1);
            let actual_alloc = aligned_alloc as *mut u8;
            
            // Calculate the total space needed including padding
            let padding = (actual_alloc as usize) - (potential_alloc as usize);
            let total_needed = min_free_size + padding;
            
            // Check if we have enough space
            let mut free_count = 0;
            let mut check_ptr = current_cell;
            while check_ptr < (self.heap_end as *mut u8) && *check_ptr == 0 {
                free_count += 1;
                check_ptr = check_ptr.add(1);
            }
            
            if free_count >= total_needed {
                // We found enough space - write the allocation metadata
                // Store the actual allocation size plus padding
                let total_size = (layout.size() as u32).to_ne_bytes();
                current_cell.add(0).write(total_size[0]);
                current_cell.add(1).write(total_size[1]);
                current_cell.add(2).write(total_size[2]);
                current_cell.add(3).write(total_size[3]);
                
                // Mark the padding bytes as non-zero to prevent them from being used
                for i in 4..padding+4 {
                    current_cell.add(i).write(0xFF);
                }
                
                return actual_alloc;
            }
            
            // Skip to next block if this one is occupied
            if *current_cell != 0 {
                let alloc_len = u32::from_ne_bytes([
                    *current_cell.add(0),
                    *current_cell.add(1),
                    *current_cell.add(2),
                    *current_cell.add(3),
                ]);
                current_cell = current_cell.add(alloc_len as usize + 4);
            } else {
                current_cell = current_cell.add(1);
            }
        }
        
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // Find the start of the metadata by working backwards
        let alloc_addr = ptr as usize;
        let metadata_alignment = core::mem::align_of::<u32>();
        let metadata_addr = ((alloc_addr - 4) & !(metadata_alignment - 1)) as *mut u8;
        
        let alloc_len = u32::from_ne_bytes([
            *metadata_addr,
            *metadata_addr.add(1),
            *metadata_addr.add(2),
            *metadata_addr.add(3),
        ]);
        
        // Zero out everything from metadata to end of allocation
        let total_size = (alloc_addr - metadata_addr as usize) + layout.size();
        for i in 0..total_size {
            metadata_addr.add(i).write(0x00);
        }
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
