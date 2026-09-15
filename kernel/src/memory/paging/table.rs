use super::{
    PageTable,
    PageTableIndices,
    ADDRESS_MASK,
    ENTRY_COUNT,
    HUGE_PAGE,
    HUGE_PAGE_SIZE,
    IDENTITY_MAP_SIZE,
    INDEX_MASK,
    PAGE_SIZE,
    PHYS_MAP_BASE,
    PRESENT,
    USER,
    WRITABLE,
};

use crate::memory::memory_map::MemoryMap;
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
    let frame =
        allocator
            .allocate_frame()
            .ok_or(())?;

    let table = unsafe {
        physical_table_pointer(
            frame.start_address,
        )?
    };

    unsafe {
        (*table).zero();
    }

    Ok((frame.start_address, table))
}

pub(super) unsafe fn allocate_table_below(
    allocator: &mut PhysicalFrameAllocator,
    limit: u64,
) -> Result<(u64, *mut PageTable), ()> {
    let frame =
        allocator
            .allocate_frame_below(limit)
            .ok_or(())?;

    let table =
        frame.start_address as *mut PageTable;

    unsafe {
        (*table).zero();
    }

    Ok((frame.start_address, table))
}

pub(super) unsafe fn physical_table_pointer(
    physical_address: u64,
) -> Result<*mut PageTable, ()> {
    if physical_address < IDENTITY_MAP_SIZE {
        return Ok(physical_address as *mut PageTable);
    }

    let virtual_address =
        PHYS_MAP_BASE
            .checked_add(physical_address)
            .ok_or(())?;

    Ok(virtual_address as *mut PageTable)
}

pub(super) unsafe fn allocate_pml4(
    allocator: &mut PhysicalFrameAllocator,
) -> Result<(u64, *mut PageTable), ()> {
    unsafe { allocate_table(allocator) }
}

pub(super) unsafe fn allocate_pml4_below(
    allocator: &mut PhysicalFrameAllocator,
    limit: u64,
) -> Result<(u64, *mut PageTable), ()> {
    unsafe {
        allocate_table_below(
            allocator,
            limit,
        )
    }
}

pub(super) unsafe fn create_page_tables(
    allocator: &mut PhysicalFrameAllocator,
) -> Result<(*mut PageTable, u64, [*mut PageTable; 4]), ()> {
    let (pml4_address, pml4) =
        unsafe {
            allocate_pml4_below(
                allocator,
                IDENTITY_MAP_SIZE,
            )?
        };

    let (pdpt_address, pdpt) =
        unsafe {
            allocate_table_below(
                allocator,
                IDENTITY_MAP_SIZE,
            )?
        };

    let mut directories =
        [core::ptr::null_mut(); 4];

    let mut directory_addresses =
        [0u64; 4];

    for index in 0..4 {
        let (address, table) =
            unsafe {
                allocate_table_below(
                    allocator,
                    IDENTITY_MAP_SIZE,
                )?
            };

        directory_addresses[index] = address;
        directories[index] = table;
    }

    unsafe {
        (*pml4).entries[0] =
            pdpt_address
            | PRESENT
            | WRITABLE;

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

    Ok((
        pml4,
        pml4_address,
        directories,
    ))
}

pub(super) unsafe fn map_physical_memory(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    memory_map: &MemoryMap<'_>,
) -> Result<(), ()> {
    let pml4_index =
        page_table_indices(PHYS_MAP_BASE).pml4;

    let pdpt = unsafe {
        ensure_table_below(
            pml4,
            pml4_index,
            allocator,
            false,
            IDENTITY_MAP_SIZE,
        )?
    };

    for index in 0..memory_map.descriptor_count() {
        let descriptor = unsafe {
            memory_map
                .descriptor(index)
                .ok_or(())?
        };

        if descriptor.ty != 7 {
            continue;
        }

        let region_size =
            descriptor
                .number_of_pages
                .checked_mul(PAGE_SIZE)
                .ok_or(())?;

        let region_end =
            descriptor
                .physical_start
                .checked_add(region_size)
                .ok_or(())?;

        map_physical_region(
            pdpt,
            allocator,
            descriptor.physical_start,
            region_end,
        )?;
    }

    Ok(())
}

unsafe fn map_physical_region(
    pdpt: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    physical_start: u64,
    physical_end: u64,
) -> Result<(), ()> {
    let start = physical_start
        .checked_add(HUGE_PAGE_SIZE - 1)
        .ok_or(())?
        & !(HUGE_PAGE_SIZE - 1);

    let end =
        physical_end
            & !(HUGE_PAGE_SIZE - 1);

    if start >= end {
        return Ok(());
    }

    let mut physical = start;

    while physical < end {
        let virtual_address =
            PHYS_MAP_BASE
                .checked_add(physical)
                .ok_or(())?;

        let indices =
            page_table_indices(
                virtual_address,
            );

        let pd = unsafe {
            ensure_table_below(
                pdpt,
                indices.pdpt,
                allocator,
                false,
                IDENTITY_MAP_SIZE,
            )?
        };

        let entry =
            unsafe {
                (*pd).entries[indices.pd]
            };

        if entry & PRESENT != 0 {
            if entry & HUGE_PAGE != 0 {
                if (entry & ADDRESS_MASK)
                    != physical
                {
                    return Err(());
                }
            } else {
                return Err(());
            }
        } else {
            unsafe {
                (*pd).entries[indices.pd] =
                    physical
                    | PRESENT
                    | WRITABLE
                    | HUGE_PAGE
                    | super::NX;
            }
        }

        physical =
            physical
                .checked_add(HUGE_PAGE_SIZE)
                .ok_or(())?;
    }

    Ok(())
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

        return unsafe {
            physical_table_pointer(
                entry & ADDRESS_MASK,
            )
        };
    }

    let frame =
        allocator
            .allocate_frame()
            .ok_or(())?;

    let table = unsafe {
        physical_table_pointer(
            frame.start_address,
        )?
    };

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

pub(super) unsafe fn ensure_table_below(
    parent: *mut PageTable,
    index: usize,
    allocator: &mut PhysicalFrameAllocator,
    user: bool,
    limit: u64,
) -> Result<*mut PageTable, ()> {
    let entry =
        unsafe { (*parent).entries[index] };

    if entry & PRESENT != 0 {
        if user && entry & USER == 0 {
            return Err(());
        }

        return Ok(
            (entry & ADDRESS_MASK)
                as *mut PageTable
        );
    }

    let frame =
        allocator
            .allocate_frame_below(limit)
            .ok_or(())?;

    let table =
        frame.start_address as *mut PageTable;

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
        unsafe {
            physical_table_pointer(
                pml4_entry & ADDRESS_MASK,
            )?
        };

    let pdpt_entry = unsafe {
        (*pdpt).entries[indices.pdpt]
    };

    if pdpt_entry & PRESENT == 0
        || pdpt_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pd =
        unsafe {
            physical_table_pointer(
                pdpt_entry & ADDRESS_MASK,
            )?
        };

    let pd_entry = unsafe {
        (*pd).entries[indices.pd]
    };

    if pd_entry & PRESENT == 0
        || pd_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pt =
        unsafe {
            physical_table_pointer(
                pd_entry & ADDRESS_MASK,
            )?
        };

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