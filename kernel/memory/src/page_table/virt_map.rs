use crate::{frame_table::FrameTable, page_table::PageTableEntry};

use super::{PageTable, PageTableIndex};

pub struct PageRegion {
    ///
    /// Page-aligned memory address
    ///
    pub address: usize,

    ///
    /// Number of pages to map
    ///
    pub count: usize,
}

///
/// Virtually map a physical
/// memory region to the virtual memory
/// linearly, this means that there is a constant
/// offset between virtual and physical memory
///
/// an identity mapping is just a virtual mapping with
/// the offset of `0`
///
/// This function will error if the desired virtual memory is not available
///
pub unsafe fn virtual_map_linear(
    page_table: &mut PageTable,
    frame_table: &mut FrameTable,
    physical_region: PageRegion,
    virtual_start_addr: usize,
) -> Result<(), LinearMapError> {
    // Number of pages an entry of each level can back
    let virt_page_num = virtual_start_addr >> 12;
    let virt_end_page = virt_page_num + physical_region.count;

    log::debug!(
        "Making sure that virtual pages {virt_page_num:#X}..{virt_end_page:#X} are mappable"
    );

    for i in virt_page_num..virt_end_page {
        let parts = decompose_virt_pageaddr(i);
        if !page_table.is_free(parts.l1_index, parts.l2_index, parts.l3_index) {
            return Err(LinearMapError::NotFree);
        }
    }

    let first_hardware_page = physical_region.address >> 12;

    for i in 0..physical_region.count {
        let hardware_page = first_hardware_page + i;
        let virtual_page = (virtual_start_addr >> 12) + i;

        let parts = decompose_virt_pageaddr(virtual_page);

        unsafe {
            page_table.virtually_map(
                frame_table,
                parts.l1_index,
                parts.l2_index,
                parts.l3_index,
                hardware_page << 12,
                PageTableEntry::FLAG_V | PageTableEntry::FLAG_R | PageTableEntry::FLAG_W | PageTableEntry::FLAG_X,
            )
        };
    }

    log::info!("Wrote all mappings");

    Ok(())
}

#[derive(Debug, Clone)]
pub enum LinearMapError {
    ///
    /// Some memory is not free to be mapped
    ///
    NotFree,
}

#[derive(Debug, Clone)]
pub struct DecomposedVirtualPageAddress {
    pub l1_index: PageTableIndex,
    pub l2_index: PageTableIndex,
    pub l3_index: PageTableIndex,
}

///
/// Decompose a page address into its indexes
///
/// NOTE: This specifically operates on 'page addresses'
///
/// this means that the 12-bit inner index should be shifted out
///
pub fn decompose_virt_pageaddr(addr: usize) -> DecomposedVirtualPageAddress {
    let l3_index = addr & 0x1FF;
    let l2_index = (addr >> 9) & 0x1FF;
    let l1_index = (addr >> 18) & 0x1FF;

    DecomposedVirtualPageAddress {
        l1_index: PageTableIndex::new(l1_index as u16),
        l2_index: PageTableIndex::new(l2_index as u16),
        l3_index: PageTableIndex::new(l3_index as u16),
    }
}
