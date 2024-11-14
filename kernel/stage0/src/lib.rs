#![no_std]

use chopin_kalloc::AllocatorVariant;
use chopin_kalloc::EarlyKernelAllocator;
use chopin_kalloc::ALLOCATOR;

extern crate alloc;

extern "C" {
    static CHOPIN_kernel_memory_end: u8;
    // static stack_top : usize;
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
    print_u64(heap_start_address);

    let early_alloc = unsafe {
        EarlyKernelAllocator::new(
            heap_start_address as usize + 32,
            heap_start_address as usize + 32 + 64_000,
        )
    }; // 64K HEAP

    unsafe {
        ALLOCATOR.allocator = AllocatorVariant::Early(early_alloc);
    }



    // let compat = device_tree.get_property("/soc/ethernet@10090000", "compatible").unwrap();

    // let compat_s = core::str::from_utf8(compat).unwrap();



    let sbi_spec_ver = sbi::base::spec_version();

    println(&alloc::format!("OPENSBI: {}.{}", sbi_spec_ver.major, sbi_spec_ver.minor));
    println(&alloc::format!("Arch: {}; Vendor: {}", sbi::base::marchid(), sbi::base::mvendorid()));
    


    let satp = riscv::register::satp::read();

    match satp.mode(){
        riscv::register::satp::Mode::Bare => {
            println("No protection")
        },
        riscv::register::satp::Mode::Sv39 => {
            println("SV39")
        },
        riscv::register::satp::Mode::Sv48 => {
            println("SV48")
        },
        riscv::register::satp::Mode::Sv57 => {
            println("SV57")
        },
        riscv::register::satp::Mode::Sv64 => {
            println("SV64")
        }
    }
    
    chopin_klog::initialize_logger();
    
    use alloc::vec::Vec;

    #[derive(Debug)]
    struct HARTInfo<'a>{
        hart_id : u32,
        status : &'a str,
        mmu : Option<&'a str>
    }

    let mut harts = Vec::with_capacity(10);

    use alloc::format;
    device_tree.enum_subnodes("/cpus").for_each(|cpu_node| {
        log::info!("CPU CORE: {cpu_node}");
        let cpu_path = format!("/cpus/{cpu_node}");

        let dev_type = device_tree.get_property(&cpu_path, "device_type").unwrap();
        let reg = device_tree.get_property(&cpu_path, "reg").unwrap();
        let status = device_tree.get_property(&cpu_path, "status").unwrap();
        let mmu_type = device_tree.get_property(&cpu_path, "mmu-type").map(|t| core::str::from_utf8(t).unwrap_or("?"));

        let status_str = core::str::from_utf8(status).unwrap();


        let hart_number = u32::from_be_bytes(reg.try_into().unwrap());

        harts.push(HARTInfo{
            hart_id: hart_number,
            status: status_str,
            mmu: mmu_type
        });
        log::info!("Pushed");
    });

    log::info!("Added all HARTs");

    for h in harts{
        log::info!("HART: {h:?}");
    }
    // let _ = harts.iter();;

    log::info!("Iterated");
    // for el in numbers {
    //     println(&alloc::format!("Number: {el}"));
    // }
    loop {}
}
