#![no_std]

use core::mem::MaybeUninit;
extern crate alloc;

pub mod frame_table;
pub mod page_table;




pub const PAGE_SIZE_B : usize = 4096;
pub const PAGE_SIZE_KB : usize = 4;




pub static mut KERNEL_FRAME_TABLE : MaybeUninit<frame_table::FrameTable> = MaybeUninit::zeroed();
pub static mut KERNEL_PAGE_TABLE : MaybeUninit<page_table::PageTable> = MaybeUninit::zeroed();
