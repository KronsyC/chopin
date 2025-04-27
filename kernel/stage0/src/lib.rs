#![no_std]

pub const PAGE_SIZE_BYTES: usize = 4096;

use alloc::vec::Vec;
use chopin_kalloc::AllocatorVariant;
use chopin_kalloc::EarlyKernelAllocator;
use chopin_kalloc::ALLOCATOR;

extern crate alloc;

extern "C" {
    static CHOPIN_kernel_memory_end: u8;
    // static stack_top : usize;
    fn CHOPIN_kern_stage0_kcore_init(hart_id: usize, value: usize) -> !;
}

fn print(s: &str) {
    for c in s.chars() {
        sbi::legacy::console_putchar(c as u8);
    }
}
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
    print("0x");
    let mut first = true;
    for i in (0..4).rev() {
        let shift = 8 * i;
        let mask = 0xFF << shift;
        let value = (v & mask) >> shift;
        let upper = ((value & 0b11110000) >> 4) as u8;
        let lower = (value & 0b00001111) as u8;

        if !first {
            sbi::legacy::console_putchar(b'_');
        }

        first = false;
        print_nibble(upper);
        print_nibble(lower);
    }
}

fn print_u64(v: u64) {
    print("0x");
    let mut first = true;
    for i in (0..8).rev() {
        let shift = 8 * i;
        let mask = 0xFF << shift;
        let value = (v & mask) >> shift;
        let upper = ((value & 0b11110000) >> 4) as u8;
        let lower = (value & 0b00001111) as u8;

        if !first {
            sbi::legacy::console_putchar(b'_');
        }
        first = false;
        print_nibble(upper);
        print_nibble(lower);
    }
}
extern "C" {
    static _start: u8;
}

#[no_mangle]
extern "C" fn CHOPIN_kern_stage0(hart_id: u32, device_tree: *const u8) -> ! {
    let start_address = unsafe { &_start as *const u8 as usize as u64 };
    let end_address = unsafe { &CHOPIN_kernel_memory_end as *const u8 as usize as u64 };
    // let address = CHOPIN_kern_stage0 as *const core::ffi::c_void;
    print("Kernel running at :: ");
    print_u64(start_address);
    println("");

    print("Kernel end at :: ");
    print_u64(end_address);
    println("");

    let kernel_region = MemoryRegion {
        addr: start_address as usize,
        size: end_address as usize - start_address as usize,
    };

    print("Init Running on HART");
    print_u32(hart_id);
    println("");

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

        print("Available memory:");
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

    let heap_start_address = unsafe { core::ptr::addr_of!(CHOPIN_kernel_memory_end) as u64 };
    print("Kernel memory ends at :: ");

    print_u64(heap_start_address);
    println("");

    let early_alloc = unsafe {
        EarlyKernelAllocator::new(
            heap_start_address as usize + 32,
            heap_start_address as usize + 32 + 64_000,
        )
    }; // 64K HEAP

    let early_heap_region = MemoryRegion {
        addr: heap_start_address as usize + 32,
        size: 64_000,
    };

    unsafe {
        ALLOCATOR.allocator = AllocatorVariant::Early(early_alloc);
    }

    println(&alloc::format!("Initiated early kernel allocator"));


    // let compat_s = core::str::from_utf8(compat).unwrap();
    let sbi_spec_ver = sbi::base::spec_version();

    println(&alloc::format!(
        "OPENSBI: {}.{}",
        sbi_spec_ver.major,
        sbi_spec_ver.minor
    ));
    println(&alloc::format!(
        "Arch: {}; Vendor: {}",
        sbi::base::marchid(),
        sbi::base::mvendorid()
    ));

    let satp = riscv::register::satp::read();

    // satp.mode
    match satp.mode() {
        riscv::register::satp::Mode::Bare => println("No protection"),
        riscv::register::satp::Mode::Sv39 => println("SV39"),
        riscv::register::satp::Mode::Sv48 => println("SV48"),
        riscv::register::satp::Mode::Sv57 => println("SV57"),
        riscv::register::satp::Mode::Sv64 => println("SV64"),
    }

    chopin_klog::initialize_logger();

    use alloc::vec::Vec;

    #[derive(Debug)]
    struct HARTInfo<'a> {
        hart_id: u32,
        status: &'a str,
        mmu: Option<&'a str>,
    }

    let mut harts = Vec::with_capacity(10);

    use alloc::format;
    device_tree.enum_subnodes("/cpus").for_each(|cpu_node| {
        let cpu_path = format!("/cpus/{cpu_node}");

        let dev_type = device_tree.get_property(&cpu_path, "device_type").unwrap();
        let reg = device_tree.get_property(&cpu_path, "reg").unwrap();
        let status = device_tree.get_property(&cpu_path, "status").unwrap();
        let mmu_type = device_tree
            .get_property(&cpu_path, "mmu-type")
            .map(|t| core::str::from_utf8(t).unwrap_or("?"));

        let status_str = core::str::from_utf8(status).unwrap();

        let hart_number = u32::from_be_bytes(reg.try_into().unwrap());

        harts.push(HARTInfo {
            hart_id: hart_number,
            status: status_str,
            mmu: mmu_type,
        });
        // log::info!("Pushed");
    });

    log::info!("Running with {} cores", harts.len());
    // println(&format!("Running with {} cores", harts.len()));

    let mut mmap = MemoryMap {
        regions: Vec::new(),
    };
    // Gather all aliases
    log::info!("DT Aliases");
    device_tree.enum_properties("/aliases").for_each(|prop| {
        // let key = format!("/aliases/{prop}");
        let val = device_tree.get_property("/aliases", prop);
        let val_s = val.map(core::str::from_utf8);
        log::info!(">: {prop} :: {val_s:?}");
    });

    log::info!("");
    log::info!("Kernel Fields");
    device_tree.enum_properties("/chosen").for_each(|sn| {
        if sn == "stdout-path" {
            let value = device_tree.get_property("/chosen", "stdout-path");

            let val_s = value.map(core::str::from_utf8);

            log::info!(">: Stdout Path: {val_s:?}");
        } else {
            log::warn!(":> Unrecognized chosen property: {sn}");
        }
    });

    let root_cellsize = {
        let size_cells = u32::from_be_bytes(
            device_tree
                .get_property("/", "#size-cells")
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
        );
        let address_cells = u32::from_be_bytes(
            device_tree
                .get_property("/", "#address-cells")
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
        );

        DTBAddressConfig {
            size_cells: size_cells as u8,
            address_cells: address_cells as u8,
        }
    };
    let soc_cellsize = {
        let size_cells = u32::from_be_bytes(
            device_tree
                .get_property("/soc", "#size-cells")
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
        );
        let address_cells = u32::from_be_bytes(
            device_tree
                .get_property("/soc", "#address-cells")
                .unwrap_or_default()
                .try_into()
                .unwrap_or_default(),
        );

        DTBAddressConfig {
            size_cells: size_cells as u8,
            address_cells: address_cells as u8,
        }
    };

    device_tree.enum_subnodes("/soc").for_each(|sn| {
        log::info!("== SOC: {sn}");

        // Extract compaibility value
        let path = format!("/soc/{sn}");
        if let Some(driver_ty) = device_tree.get_property(&path, "compatible") {
            let dt_s = core::str::from_utf8(driver_ty);
            let compat = dt_s.map(CompatibleEntry::parse_many);
            log::info!(">: Compatible with driver: {compat:?}");
        } else {
            log::warn!(">: No compatible driver type found");
        }

        // Extract unique Handle Value
        if let Some(phandle) = device_tree.get_property(&path, "phandle") {
            let val = u32::from_be_bytes(phandle.try_into().unwrap_or_default());
            log::info!(">: PHandle: {val:#08X}");
        } else {
            log::warn!(">: No phandle");
        }

        if let Some(reg) = device_tree.get_property(&path, "reg") {
            let addrs = soc_cellsize.interpret_reg(reg);
            log::info!(">: MMIO ADDRS");
            for addr in addrs.iter() {
                log::info!(">: :: At {:#X} [{:#X} bytes]", addr.0, addr.1);
            }
        } else {
            log::warn!(">: No mem addrs");
        }

        device_tree.enum_properties(&path).for_each(|sn| {
            log::info!(">> {sn}");
        });
    });

    log::info!("");
    log::info!("Memory Devices ::");
    device_tree.enum_subnodes("/").for_each(|sn| {
        if sn.starts_with("memory@") {
            log::info!("Found memory node: {sn}");
            let path = format!("/{sn}");

            if let Some(reg) = device_tree.get_property(&path, "reg") {
                let addrs = root_cellsize.interpret_reg(reg);
                log::info!(">: Got mem addrs");
                for addr in addrs.iter() {
                    mmap.add_region(MemoryRegion {
                        addr: addr.0 as usize,
                        size: addr.1 as usize,
                    });
                    log::info!(">: :: At {:#X} [{:#X} bytes]", addr.0, addr.1);
                }
            }
            // device_tree.enum_properties(&path).for_each(|p| {
            //     log::info!(">: {p}");
            // });
        }
    });

    // log::info!("");
    // log::info!("Reserved Memory ::");
    device_tree
        .enum_subnodes("/reserved-memory")
        .for_each(|sn| {
            let path = format!("/reserved-memory/{sn}");
            log::info!(">: {sn}");
            if let Some(reg) = device_tree.get_property(&path, "reg") {
                let addrs = root_cellsize.interpret_reg(reg);
                // log::info!(">: Got mem addrs");
                for addr in addrs.iter() {
                    // log::info!(">: :: At {:#X} [{:#X} bytes]", addr.0, addr.1);

                    mmap.cut_region(MemoryRegion {
                        addr: addr.0 as usize,
                        size: addr.1 as usize,
                    });
                }
            }

            // device_tree.enum_properties(&path).for_each(|p| {
            //     log::info!("::::: {p}");
            // });
        });

    // Save the kernel and heap
    mmap.cut_region(kernel_region);
    mmap.cut_region(early_heap_region);

    // device_tree.enum_subnodes("/").for_each(|sn| {
    //     log::info!("SN: {sn}");
    // });
    // device_tree.enum_properties("/").for_each(|sn| {
    //     log::info!("SN: {sn}");
    // });

    // device_tree.enum_properties("/soc")
    //     .for_each(|n| {
    //         log::info!("Subnode: {n:?}");
    //     });

    log::info!("Memory Map: {mmap:?}");

    // let mut root_page_table = mmap
    //     .bite_first_aligned(PAGE_SIZE_BYTES, PAGE_SIZE_BYTES)
    //     .expect("Failed to allocate root page table");
    //
    // unsafe {
    //     root_page_table.zero();
    // };
    // log::info!("Root page table lives at: {root_page_table:?}");
    //
    // let pt_entries = unsafe { root_page_table.as_slice::<PageTableEntry>() };

    // let c = pt_entries.len();
    // log::info!("PT has {c} entries");

    log::info!("Constructing the frame table");

    let mut ft = chopin_memory::frame_table::FrameTable {
        segments: Vec::new(),
    };

    for region in mmap.regions {
        let segment =
            chopin_memory::frame_table::FrameSegment::initialize(region.addr, region.size);

        match segment {
            Some(s) => {
                log::info!("Initialized frame segment at {region:?}");
                ft.segments.push(s);
            }
            None => {
                log::warn!("Memory region too small to create segment: {region:?}");
            }
        }
        // ft.segmens.push(segment);
    }

    log::info!("Constructed frame table");

    let root_page_table = ft
        .alloc_front(1, chopin_memory::frame_table::FrameState::PageTable, 0)
        .unwrap();

    unsafe { root_page_table.zero() };

    log::info!("Root PT at {root_page_table:?}");

    let entries =
        unsafe { root_page_table.as_slice::<chopin_memory::page_table::PageTableEntry>() };

    unsafe {
        chopin_memory::KERNEL_PAGE_TABLE
            .write(chopin_memory::page_table::PageTable { entries })
    };
    unsafe { chopin_memory::KERNEL_FRAME_TABLE.write(ft) };
    // for pte in pt_entries{
    //     let k = pte.kind();
    //     log::info!("PTK: {k:?}");
    // }

    loop {}

    // Allocate stack space for every hart we have
}

#[derive(Debug, Clone)]
pub struct CompatibleEntry<'a> {
    pub manufacturer: &'a str,
    pub model: &'a str,
}

impl<'a> CompatibleEntry<'a> {
    pub fn parse_many(value: &'a str) -> Vec<Self> {
        let mut rem = value;

        let mut r = Vec::new();

        loop {
            let (v, rest) = Self::parse(rem);

            r.push(v);

            if let Some(rest) = rest {
                rem = rest
            } else {
                break;
            }
        }

        r
    }

    ///
    /// Parse a `compatible entry`
    ///
    /// this will consume an entry and return the rest of the stream, if
    /// more than one are detected
    ///
    pub fn parse(value: &'a str) -> (Self, Option<&'a str>) {
        if let Some(null_index) = value.find('\0') {
            let (relevant, remainder) = if null_index == value.len() - 1 {
                (&value[0..null_index], None)
            } else {
                (&value[0..null_index], Some(&value[null_index + 1..]))
            };

            if let Some((manu, model)) = relevant.split_once(',') {
                (
                    CompatibleEntry {
                        manufacturer: manu,
                        model,
                    },
                    remainder,
                )
            } else {
                (
                    CompatibleEntry {
                        manufacturer: "Malformed Input",
                        model: "Malformed Input",
                    },
                    remainder,
                )
            }
        } else {
            // Malformed entry
            (
                CompatibleEntry {
                    manufacturer: "",
                    model: "",
                },
                None,
            )
        }
    }
}

pub struct DTBAddressConfig {
    pub address_cells: u8,
    pub size_cells: u8,
}

impl DTBAddressConfig {
    pub fn interpret_reg(&self, data: &[u8]) -> Vec<(u128, u128)> {
        let split_size = (self.address_cells as usize + self.size_cells as usize) * 4;

        data.chunks_exact(split_size)
            .map(|cnk| {
                let (addr, size) = cnk.split_at(self.address_cells as usize * 4);

                (numbify(addr), numbify(size))
            })
            .collect()
    }
}

///
/// Big-Endian slice to u128
///
fn numbify(slice: &[u8]) -> u128 {
    let mut v = 0u128;

    for el in slice.iter() {
        v <<= 8;
        v |= *el as u128;
    }

    v
}

#[derive(Clone)]
pub struct MemoryRegion {
    pub addr: usize,
    pub size: usize,
}

impl core::fmt::Debug for MemoryRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MRegion [ at = {:#X}, len = {:#X} ]",
            self.addr, self.size
        )
    }
}

impl MemoryRegion {
    pub fn overlaps(&self, other: &MemoryRegion) -> bool {
        self.addr < other.end() && other.addr < self.end()
    }

    pub fn end(&self) -> usize {
        self.addr + self.size
    }

    pub fn extend_with(&mut self, other: &MemoryRegion) {
        let new_start = self.addr.min(other.addr);
        let new_end = self.end().max(other.end());
        self.addr = new_start;
        self.size = new_end - new_start;
    }

    /// # Safety
    ///
    /// - `addr` must be valid and mapped in the current address space
    /// - Memory must be initialized to `T`-compatible values
    /// - Caller must ensure correct lifetime management
    pub unsafe fn as_slice<T>(&self) -> &'static mut [T] {
        let ptr = self.addr as *mut T;
        let count = self.size / core::mem::size_of::<T>();
        core::slice::from_raw_parts_mut(ptr, count)
    }

    ///
    /// Zero out the memory region
    ///
    /// # Safety
    ///
    /// this region must be writable memory
    ///
    pub unsafe fn zero(&mut self) {
        let ptr = self.addr as *mut u8;

        core::ptr::write_bytes(ptr, 0, self.size);
    }
}

#[derive(Debug, Clone)]
pub struct MemoryMap {
    /// Memory Regions — non-overlapping, sorted by start address
    pub regions: Vec<MemoryRegion>,
}

impl MemoryMap {
    pub fn add_region(&mut self, mut r: MemoryRegion) {
        if self.regions.is_empty() {
            self.regions.push(r);
            return;
        }

        let mut i = 0;
        while i < self.regions.len() {
            let current = &self.regions[i];

            // Merge or grow if overlaps or adjacent
            if current.overlaps(&r) || current.end() == r.addr || r.end() == current.addr {
                let mut new_region = self.regions.remove(i);
                new_region.extend_with(&r);
                r = new_region;

                // Keep merging forward if more overlaps happen
                while i < self.regions.len()
                    && (self.regions[i].overlaps(&r)
                        || self.regions[i].addr == r.end()
                        || self.regions[i].end() == r.addr)
                {
                    let next = self.regions.remove(i);
                    r.extend_with(&next);
                }

                self.regions.insert(i, r);
                return;
            }

            i += 1;
        }

        // No overlap, add as new
        self.regions.push(r);
        self.regions.sort_by_key(|v| v.addr);
    }

    ///
    /// Bite out the first memory region of that size
    ///
    pub fn bite_first(&mut self, size_bytes: usize) -> Option<MemoryRegion> {
        for r in &mut self.regions {
            if r.size >= size_bytes {
                // We have enough space in this region,
                // trim the start

                let start = r.addr;
                r.addr += size_bytes;
                r.size -= size_bytes;

                return Some(MemoryRegion {
                    addr: start,
                    size: size_bytes,
                });
            }
        }

        None
    }
    ///
    /// Bite out the first memory region of the given size and alignment
    ///
    pub fn bite_first_aligned(&mut self, size_bytes: usize, align: usize) -> Option<MemoryRegion> {
        for i in 0..self.regions.len() {
            let region = &mut self.regions[i];

            // Find aligned start within this region
            let aligned_start = align_up(region.addr, align);
            let end = region.addr + region.size;

            // Check if there's enough space from aligned_start
            if aligned_start + size_bytes <= end {
                // How much we're trimming from the start
                let trim = aligned_start - region.addr;

                // Adjust original region
                region.addr = aligned_start + size_bytes;
                region.size = end - region.addr;

                // If we trimmed all of it, remove the region
                if region.size == 0 {
                    self.regions.remove(i);
                }

                return Some(MemoryRegion {
                    addr: aligned_start,
                    size: size_bytes,
                });
            }
        }

        None
    }

    pub fn cut_region(&mut self, cut: MemoryRegion) {
        let mut new_regions = Vec::new();

        for region in self.regions.drain(..) {
            if !region.overlaps(&cut) {
                new_regions.push(region);
                continue;
            }

            let r_start = region.addr;
            let r_end = region.end();
            let c_start = cut.addr;
            let c_end = cut.end();

            // Left slice before cut
            if c_start > r_start {
                new_regions.push(MemoryRegion {
                    addr: r_start,
                    size: c_start - r_start,
                });
            }

            // Right slice after cut
            if c_end < r_end {
                new_regions.push(MemoryRegion {
                    addr: c_end,
                    size: r_end - c_end,
                });
            }
        }

        self.regions = new_regions;
        self.regions.sort_by_key(|v| v.addr);
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
