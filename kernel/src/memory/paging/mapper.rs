use super::{
    table::{ensure_table, find_page_table, page_table_indices},
    PageFlags,
    PageTable,
    ADDRESS_MASK,
    HUGE_PAGE,
    PAGE_SIZE,
    PRESENT,
    PCD,
    USER,
    WRITABLE,
};
use crate::memory::physical::PhysicalFrameAllocator;

pub(super) fn flags_to_entry(flags: PageFlags) -> u64 {
    PRESENT
        | if flags.writable { WRITABLE } else { 0 }
        | if flags.cache_disable { PCD } else { 0 }
        | if flags.user { USER } else { 0 }
}

fn validate_mapping(
    virtual_address: u64,
    physical_address: u64,
    flags: PageFlags,
) -> Result<(), ()> {
    if !super::is_canonical_address(virtual_address)
        || virtual_address & (PAGE_SIZE - 1) != 0
    {
        return Err(());
    }

    if physical_address & (PAGE_SIZE - 1) != 0
        || !is_valid_physical_address(physical_address)
    {
        return Err(());
    }

    if flags.user
        && !super::is_user_address(virtual_address)
    {
        return Err(());
    }

    Ok(())
}

pub unsafe fn map_page(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    virtual_address: u64,
    physical_address: u64,
    flags: PageFlags,
) -> Result<(), ()> {
    validate_mapping(
        virtual_address,
        physical_address,
        flags,
    )?;

    let indices =
        page_table_indices(virtual_address);

    let user = flags.user;

    let pdpt = unsafe {
        ensure_table(
            pml4,
            indices.pml4,
            allocator,
            user,
        )?
    };

    let pd = unsafe {
        ensure_table(
            pdpt,
            indices.pdpt,
            allocator,
            user,
        )?
    };

    let pd_entry = unsafe {
        (*pd).entries[indices.pd]
    };

    if pd_entry & PRESENT != 0
        && pd_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pt = unsafe {
        ensure_table(
            pd,
            indices.pd,
            allocator,
            user,
        )?
    };

    unsafe {
        if (*pt).entries[indices.pt] & PRESENT != 0 {
            return Err(());
        }

        (*pt).entries[indices.pt] =
            physical_address
            | flags_to_entry(flags);
    }

    Ok(())
}

pub unsafe fn unmap_page(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Result<u64, ()> {
    let (pt, indices) = unsafe {
        find_page_table(
            pml4,
            virtual_address,
        )?
    };

    let pte = unsafe {
        (*pt).entries[indices.pt]
    };

    if pte & PRESENT == 0 {
        return Err(());
    }

    let physical_address =
        pte & ADDRESS_MASK;

    unsafe {
        (*pt).entries[indices.pt] = 0;

        core::arch::asm!(
            "invlpg [{}]",
            in(reg) virtual_address,
            options(nostack, preserves_flags)
        );
    }

    Ok(physical_address)
}

fn is_valid_physical_address(
    address: u64,
) -> bool {
    (address >> 52) == 0
}