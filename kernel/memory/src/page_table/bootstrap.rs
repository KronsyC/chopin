use crate::frame_table::FrameTable;
use crate::page_table::{PTEKind, PageTable, PageTableEntry, PageTableIndex};

///
/// Bootstrap the page table
///
/// this operation will simply create an initial mapping
/// in the page table, which references itself
///
/// this is a critical operation in the virtmem init process
///
/// NOTE:
///
/// This MUST be the first operation used on the page table
///
pub unsafe fn bootstrap_pt(root_page_table: &mut PageTable, frame_table: &mut FrameTable) -> PageTableOffsettingData {
    log::info!("Bootstapping page table entries");
    // We start the page table at
    // 280, 0, 0

    let mut inter_1 = root_page_table
        .allocate_intermediary(PageTableIndex::new(280), frame_table)
        .unwrap();
    let mut inter_2 = inter_1
        .page_table
        .allocate_intermediary(PageTableIndex::new(0), frame_table)
        .unwrap();

    log::info!("Allocated bootstrap self-ref chain parents");

    // inter_2 contains all the actual page table links to real memory, occupy this by mapping all
    // the memory that was taken from the frame_table at the front

    // We can basically guarantee there are no more than 512 allocation at this time

    frame_table
        .iter_front()
        .enumerate()
        .for_each(|(index, address)| {
            log::debug!("Placing {address:#016X} at child #{index}");

            // We have all the page-table addresses we want to map in

            inter_2.page_table.make_mapping(
                PageTableIndex::new(index as u16),
                address,
                PageTableEntry::FLAG_V | PageTableEntry::FLAG_R | PageTableEntry::FLAG_W,
            );
        });

    PageTableOffsettingData{
        physical_root_null_offset: root_page_table as *mut PageTable as usize,
        virtual_root_null_offset: PT_VIRT_START
    }
}

pub const PT_VIRT_START : usize = 0b1111111111111111111111111_100011000_000000000_000000000_000000000000;

pub struct PageTableOffsettingData{
    pub physical_root_null_offset : usize,
    pub virtual_root_null_offset : usize
}
