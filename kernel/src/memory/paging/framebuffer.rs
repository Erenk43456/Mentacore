use super::{
    table::allocate_table,
    PageTable,
    ENTRY_COUNT,
    HUGE_PAGE_SIZE,
    IDENTITY_MAP_SIZE,
    PAGE_SIZE,
    PCD,
    PRESENT,
    WRITABLE,
};
use crate::memory::physical::PhysicalFrameAllocator;
use mentacore_boot_protocol::BootInfo;

pub(super) fn map_framebuffer(
    pd2: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) -> Result<(), ()> {
    let framebuffer_start = boot_info.framebuffer_addr;
    let framebuffer_size = boot_info.framebuffer_size;

    if framebuffer_start == 0
        || framebuffer_size == 0
    {
        return Err(());
    }

    let framebuffer_end =
        framebuffer_start
            .checked_add(framebuffer_size)
            .ok_or(())?;

    if framebuffer_start >= IDENTITY_MAP_SIZE
        || framebuffer_end > IDENTITY_MAP_SIZE
        || framebuffer_end <= framebuffer_start
    {
        return Err(());
    }

    let first_huge_page =
        framebuffer_start / HUGE_PAGE_SIZE;

    let last_huge_page =
        (framebuffer_end - 1) / HUGE_PAGE_SIZE;

    for huge_page_index
        in first_huge_page..=last_huge_page
    {
        let pd_index =
            (huge_page_index
                % ENTRY_COUNT as u64) as usize;

        let (page_table_address, page_table) =
            unsafe { allocate_table(allocator)? };

        unsafe {
            (*pd2).entries[pd_index] =
                page_table_address
                | PRESENT
                | WRITABLE;
        }

        let huge_page_start =
            huge_page_index * HUGE_PAGE_SIZE;

        for pt_index in 0..ENTRY_COUNT {
            let page_start =
                huge_page_start
                + (pt_index as u64) * PAGE_SIZE;

            if page_start < framebuffer_start
                || page_start >= framebuffer_end
            {
                continue;
            }

            unsafe {
                (*page_table).entries[pt_index] =
                    page_start
                    | PRESENT
                    | WRITABLE
                    | PCD;
            }
        }
    }

    Ok(())
}