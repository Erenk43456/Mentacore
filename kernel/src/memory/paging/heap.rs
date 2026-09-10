use super::{
    table::{ensure_table, page_table_indices},
    PageTable,
    HUGE_PAGE,
    PAGE_SIZE,
    PRESENT,
};
use crate::memory::physical::PhysicalFrameAllocator;

pub(super) fn map_heap(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    virtual_start: u64,
    size: u64,
) -> Result<(), ()> {
    if size == 0
        || virtual_start & (PAGE_SIZE - 1) != 0
    {
        return Err(());
    }

    let virtual_end =
        virtual_start.checked_add(size).ok_or(())?;

    if virtual_end <= virtual_start
        || !super::is_canonical_address(virtual_start)
        || !super::is_canonical_address(virtual_end - 1)
    {
        return Err(());
    }

    let start =
        page_table_indices(virtual_start);

    let end =
        page_table_indices(virtual_end - 1);

    if start.pdpt != end.pdpt {
        return Err(());
    }

    let pdpt = unsafe {
        ensure_table(
            pml4,
            start.pml4,
            allocator,
            false,
        )?
    };

    let pd = unsafe {
        let entry =
            (*pdpt).entries[start.pdpt];

        if entry & PRESENT != 0
            && entry & HUGE_PAGE != 0
        {
            return Err(());
        }

        ensure_table(
            pdpt,
            start.pdpt,
            allocator,
            false,
        )?
    };

    for pd_index in start.pd..=end.pd {
        let entry = unsafe {
            (*pd).entries[pd_index]
        };

        if entry & PRESENT != 0
            && entry & HUGE_PAGE != 0
        {
            return Err(());
        }

        unsafe {
            ensure_table(
                pd,
                pd_index,
                allocator,
                false,
            )?;
        }
    }

    Ok(())
}