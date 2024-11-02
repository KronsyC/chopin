#![no_std]

extern crate alloc;

///
/// Simple chopin memory allocator
/// just does heap scans
///
/// Every allocation has a leading u32
/// representing its length
///
struct EarlyKernelAllocator {
    heap_start: usize,
    heap_end: usize,
}

impl EarlyKernelAllocator {
    unsafe fn new(start: usize, end: usize) -> EarlyKernelAllocator {
        println("Alloc begins at:");
        print_u64(start as u64);
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

        println("Within alloc");
        let min_free_size = layout.size() + 4;

        let mut current_cell = self.heap_start as *mut u8;

        loop {
            if current_cell >= self.heap_end as *mut u8 {
                println("alloc failed");
                return core::ptr::null_mut();
            } else if *current_cell == 0 {
                // Check if this block has enough bytes

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

                    println("Making alloc");
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

enum AllocatorVariant {
    None,
    Early(EarlyKernelAllocator),
}

struct KernelAllocator {
    allocator: AllocatorVariant,
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

extern "C" {
    static CHOPIN_kernel_memory_end: u8;
    // static stack_top : usize;
}

#[global_allocator]
static mut ALLOCATOR: KernelAllocator = KernelAllocator {
    allocator: AllocatorVariant::None,
};
fn println(s: &str) {
    for c in s.chars() {
        sbi::legacy::console_putchar(c as u8);
    }
    sbi::legacy::console_putchar(b'\n');
}

fn print_nibble(val: u8) {
    let table = b"0123456789ABCDEF";
    let c = table[val as usize];
    sbi::legacy::console_putchar(c);
}

fn print_u32(v: u32) {
    for i in (0..4).rev() {
        let shift = 8 * i;
        let mask = 0xFF << shift;
        let value = (v & mask) >> shift;
        let upper = ((value & 0b11110000) >> 4) as u8;
        let lower = (value & 0b00001111) as u8;

        sbi::legacy::console_putchar(b':');
        print_nibble(upper);
        print_nibble(lower);
    }
}

fn print_u64(v: u64) {
    for i in (0..8).rev() {
        let shift = 8 * i;
        let mask = 0xFF << shift;
        let value = (v & mask) >> shift;
        let upper = ((value & 0b11110000) >> 4) as u8;
        let lower = (value & 0b00001111) as u8;

        sbi::legacy::console_putchar(b':');
        print_nibble(upper);
        print_nibble(lower);
    }
}
#[no_mangle]
extern "C" fn CHOPIN_kern_stage0(hart_id: u32, device_tree: *const u8) -> ! {
    println("CHOPIN Bootloader :: Stage0");

    // The second u32 in the header contains the total size

    let device_tree =
        unsafe { hermit_dtb::Dtb::from_raw(device_tree) }.expect("Failed to load DTB");

    println("Loaded DTB");

    let address_cells = device_tree.get_property("/", "#address-cells").unwrap();
    let size_cells = device_tree.get_property("/", "#size-cells").unwrap();
    let rsv_address_cells = device_tree
        .get_property("/reserved-memory", "#address-cells")
        .unwrap();
    let rsv_size_cells = device_tree
        .get_property("/reserved-memory", "#size-cells")
        .unwrap();

    let address_cells = u32::from_be_bytes(address_cells.try_into().unwrap());
    let size_cells = u32::from_be_bytes(size_cells.try_into().unwrap());
    let rsv_address_cells = u32::from_be_bytes(rsv_address_cells.try_into().unwrap());
    let rsv_size_cells = u32::from_be_bytes(rsv_size_cells.try_into().unwrap());
    assert_eq!(
        address_cells, size_cells,
        "Chopin only supports this configuration"
    );
    assert_eq!(
        rsv_address_cells, size_cells,
        "Chopin only supports this configuration"
    );
    assert_eq!(
        rsv_size_cells, size_cells,
        "Chopin only supports this configuration"
    );
    assert_eq!(address_cells, 2, "Chopin only supports 64 bit systems");

    let mem_dev_type = device_tree.get_property("/memory@", "device_type").unwrap();
    let mem_registry = device_tree.get_property("/memory@", "reg").unwrap();

    println("Memory Device Type:");
    println(core::str::from_utf8(mem_dev_type).unwrap());

    mem_registry.chunks(16).for_each(|c| {
        let address = u64::from_be_bytes(c[0..8].try_into().unwrap());
        let size = u64::from_be_bytes(c[8..16].try_into().unwrap());

        println("Available memory:");
        print_u64(address);
        println("\n ");
        print_u64(size);
        println("");
    });

    let reserved_ranges = device_tree
        .get_property("/reserved-memory", "ranges")
        .unwrap();

    assert_eq!(reserved_ranges.len(), 0);
    reserved_ranges.chunks(16).for_each(|r| {
        let address = u64::from_be_bytes(r[0..8].try_into().unwrap());
        let size = u64::from_be_bytes(r[8..16].try_into().unwrap());

        println("Reserved memory:");
        print_u64(address);
        println("\n ");
        print_u64(size);
        println("");
    });

    device_tree
        .enum_properties("/reserved-memory")
        .for_each(|s| {
            println(s);
        });

    println("Kernel memory ends at");

    let heap_start_address = unsafe { core::ptr::addr_of!(CHOPIN_kernel_memory_end) as u64 };
    unsafe {
        print_u64(heap_start_address);
    }

    let early_alloc = unsafe {
        EarlyKernelAllocator::new(
            heap_start_address as usize + 32,
            heap_start_address as usize + 32 + 64_000,
        )
    }; // 64K HEAP

    println("Switching in allocator\n");
    unsafe {
        ALLOCATOR.allocator = AllocatorVariant::Early(early_alloc);
    }

    println("allocating");

    let numbers = alloc::vec![1, 2, 3, 4, 5];

    println("allocated");

    for el in numbers {
        println(&alloc::format!("Number: {el}"));
    }
    loop {}
}
