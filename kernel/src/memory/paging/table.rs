use super::{
    PageTable,
    PageTableIndices,
    ADDRESS_MASK,
    ENTRY_COUNT,
    HUGE_PAGE,
    INDEX_MASK,
    PAGE_SIZE,
    PRESENT,
    USER,
    WRITABLE,
};
use crate::memory::physical::PhysicalFrameAllocator;

impl PageTable {
    pub(super) fn zero(&mut self) {
        for entry in &mut self.entries {
            *entry = 0;
        }
    }
}

pub(super) fn page_table_indices(address: u64) -> PageTableIndices {
    PageTableIndices {
        pml4: ((address >> 39) & INDEX_MASK) as usize,
        pdpt: ((address >> 30) & INDEX_MASK) as usize,
        pd: ((address >> 21) & INDEX_MASK) as usize,
        pt: ((address >> 12) & INDEX_MASK) as usize,
    }
}

pub(super) unsafe fn allocate_table(
    allocator: &mut PhysicalFrameAllocator,
) -> Result<(u64, *mut PageTable), ()> {
    let frame = allocator.allocate_frame().ok_or(())?;
    let table = frame.start_address as *mut PageTable;

    unsafe {
        (*table).zero();
    }

    Ok((frame.start_address, table))
}

pub(super) unsafe fn create_page_tables(
    allocator: &mut PhysicalFrameAllocator,
) -> Result<(*mut PageTable, u64, [*mut PageTable; 4]), ()> {
    let (pml4_address, pml4) =
        unsafe { allocate_table(allocator)? };

    let (pdpt_address, pdpt) =
        unsafe { allocate_table(allocator)? };

    let mut directories = [core::ptr::null_mut(); 4];
    let mut directory_addresses = [0u64; 4];

    for index in 0..4 {
        let (address, table) =
            unsafe { allocate_table(allocator)? };

        directory_addresses[index] = address;
        directories[index] = table;
    }

    unsafe {
        (*pml4).entries[0] =
            pdpt_address | PRESENT | WRITABLE;

        for index in 0..4 {
            (*pdpt).entries[index] =
                directory_addresses[index]
                | PRESENT
                | WRITABLE;
        }
    }

    for directory in directories {
        fill_identity_directory(directory);
    }

    Ok((pml4, pml4_address, directories))
}

pub(super) unsafe fn ensure_table(
    parent: *mut PageTable,
    index: usize,
    allocator: &mut PhysicalFrameAllocator,
    user: bool,
) -> Result<*mut PageTable, ()> {
    let entry = unsafe { (*parent).entries[index] };

    if entry & PRESENT != 0 {
        if user && entry & USER == 0 {
            return Err(());
        }

        return Ok(
            (entry & ADDRESS_MASK) as *mut PageTable
        );
    }

    let frame = allocator.allocate_frame().ok_or(())?;
    let table = frame.start_address as *mut PageTable;

    unsafe {
        (*table).zero();

        (*parent).entries[index] =
            frame.start_address
            | PRESENT
            | WRITABLE
            | if user { USER } else { 0 };
    }

    Ok(table)
}

pub(super) unsafe fn find_page_table(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Result<(*mut PageTable, PageTableIndices), ()> {
    if !super::is_canonical_address(virtual_address)
        || virtual_address & (PAGE_SIZE - 1) != 0
    {
        return Err(());
    }

    let indices = page_table_indices(virtual_address);

    let pml4_entry = unsafe {
        (*pml4).entries[indices.pml4]
    };

    if pml4_entry & PRESENT == 0 {
        return Err(());
    }

    let pdpt =
        (pml4_entry & ADDRESS_MASK) as *mut PageTable;

    let pdpt_entry = unsafe {
        (*pdpt).entries[indices.pdpt]
    };

    if pdpt_entry & PRESENT == 0
        || pdpt_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pd =
        (pdpt_entry & ADDRESS_MASK) as *mut PageTable;

    let pd_entry = unsafe {
        (*pd).entries[indices.pd]
    };

    if pd_entry & PRESENT == 0
        || pd_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pt =
        (pd_entry & ADDRESS_MASK) as *mut PageTable;

    Ok((pt, indices))
}

fn fill_identity_directory(
    page_directory: *mut PageTable,
) {
    for index in 0..ENTRY_COUNT {
        let physical_address =
            (index as u64) * super::HUGE_PAGE_SIZE;

        unsafe {
            (*page_directory).entries[index] =
                physical_address
                | PRESENT
                | WRITABLE
                | HUGE_PAGE;
        }
    }
}