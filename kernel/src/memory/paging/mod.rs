mod address_space;
mod framebuffer;
mod heap;
mod mapper;
mod registers;
mod table;

use super::heap::{HEAP_SIZE, HEAP_START};
use super::physical::PhysicalFrameAllocator;
use mentacore_boot_protocol::BootInfo;

pub const PAGE_SIZE: u64 = 4096;

pub(super) const HUGE_PAGE_SIZE: u64 = 0x20_0000;
pub(super) const ENTRY_COUNT: usize = 512;

pub(super) const PRESENT: u64 = 1 << 0;
pub(super) const WRITABLE: u64 = 1 << 1;
pub(super) const USER: u64 = 1 << 2;
pub(super) const PCD: u64 = 1 << 4;
pub(super) const HUGE_PAGE: u64 = 1 << 7;

pub(super) const ADDRESS_MASK: u64 =
    0x000f_ffff_ffff_f000;

pub(super) const INDEX_MASK: u64 = 0x1ff;

pub(super) const IDENTITY_MAP_SIZE: u64 =
    0x1_0000_0000;

pub const USER_SPACE_START: u64 =
    0x0000_0000_0000_0000;

pub const USER_SPACE_END: u64 =
    0x0000_7FFF_FFFF_FFFF;

pub const KERNEL_SPACE_START: u64 =
    0xFFFF_8000_0000_0000;

pub const KERNEL_SPACE_END: u64 =
    0xFFFF_FFFF_FFFF_FFFF;

#[derive(Clone, Copy)]
pub struct PageFlags {
    pub writable: bool,
    pub cache_disable: bool,
    pub user: bool,
}

pub(super) const KERNEL_FLAGS: PageFlags =
    PageFlags {
        writable: true,
        cache_disable: false,
        user: false,
    };

#[repr(align(4096))]
pub struct PageTable {
    pub(super) entries: [u64; ENTRY_COUNT],
}

#[derive(Clone, Copy)]
pub(super) struct PageTableIndices {
    pub(super) pml4: usize,
    pub(super) pdpt: usize,
    pub(super) pd: usize,
    pub(super) pt: usize,
}

pub(super) fn is_canonical_address(
    address: u64,
) -> bool {
    let sign_bit = (address >> 47) & 1;
    let upper_bits = address >> 48;

    if sign_bit == 0 {
        upper_bits == 0
    } else {
        upper_bits == 0xFFFF
    }
}

pub fn is_user_address(address: u64) -> bool {
    address >= USER_SPACE_START
        && address <= USER_SPACE_END
}

pub fn is_kernel_address(address: u64) -> bool {
    address >= KERNEL_SPACE_START
        && address <= KERNEL_SPACE_END
}

pub unsafe fn init(
    allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) -> Result<(), ()> {
    let (pml4, pml4_address, directories) =
        unsafe { table::create_page_tables(allocator)? };

    let pd2 = directories[2];

    framebuffer::map_framebuffer(
        pd2,
        allocator,
        boot_info,
    )?;

    unsafe {
        validate_initial_mapping_rejections(
            pml4,
            allocator,
        )?;
    }

    let test_virtual =
        0xFFFF_9000_0000_0000;

    let test_frame =
        allocator.allocate_frame().ok_or(())?;

    unsafe {
        mapper::map_page(
            pml4,
            allocator,
            test_virtual,
            test_frame.start_address,
            KERNEL_FLAGS,
        )?;
    }

    heap::map_heap(
        pml4,
        allocator,
        HEAP_START,
        HEAP_SIZE,
    )?;

    unsafe {
        registers::load_cr3(pml4_address);
    }

    unsafe {
        validate_initial_mapping(
            pml4,
            allocator,
            test_virtual,
        )?;
    }

    Ok(())
}

unsafe fn validate_initial_mapping_rejections(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
) -> Result<(), ()> {
    let invalid_virtual =
        0x0000_8000_0000_0000;

    let invalid_frame =
        allocator.allocate_frame().ok_or(())?;

    unsafe {
        if mapper::map_page(
            pml4,
            allocator,
            invalid_virtual,
            invalid_frame.start_address,
            KERNEL_FLAGS,
        )
        .is_ok()
        {
            return Err(());
        }
    }

    let invalid_physical =
        0x0010_0000_0000_0000;

    unsafe {
        if mapper::map_page(
            pml4,
            allocator,
            0xFFFF_9000_0000_2000,
            invalid_physical,
            KERNEL_FLAGS,
        )
        .is_ok()
        {
            return Err(());
        }
    }

    Ok(())
}

unsafe fn validate_initial_mapping(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    virtual_address: u64,
) -> Result<(), ()> {
    let pointer =
        virtual_address as *mut u64;

    unsafe {
        pointer.write(
            0xDEAD_BEEF_CAFE_BABE
        );

        if pointer.read()
            != 0xDEAD_BEEF_CAFE_BABE
        {
            return Err(());
        }
    }

    let frame =
        allocator.allocate_frame().ok_or(())?;

    unsafe {
        mapper::map_page(
            pml4,
            allocator,
            virtual_address + PAGE_SIZE,
            frame.start_address,
            KERNEL_FLAGS,
        )?;
    }

    let pointer =
        (virtual_address + PAGE_SIZE)
            as *mut u64;

    unsafe {
        pointer.write(
            0x1122_3344_5566_7788
        );

        if pointer.read()
            != 0x1122_3344_5566_7788
        {
            return Err(());
        }
    }

    Ok(())
}

#[cfg(feature = "kernel-tests")]
pub unsafe fn test_page_table_empty(
    page_table: *mut PageTable,
) -> bool {
    unsafe {
        for entry in &(*page_table).entries {
            if *entry != 0 {
                return false;
            }
        }
    }

    true
}

#[cfg(feature = "kernel-tests")]
pub unsafe fn test_entry(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Option<[u64; 4]> {
    if !is_canonical_address(virtual_address) {
        return None;
    }

    let indices =
        table::page_table_indices(virtual_address);

    let pml4_entry = unsafe {
        (*pml4).entries[indices.pml4]
    };

    if pml4_entry & PRESENT == 0 {
        return None;
    }

    let pdpt =
        (pml4_entry & ADDRESS_MASK)
            as *mut PageTable;

    let pdpt_entry = unsafe {
        (*pdpt).entries[indices.pdpt]
    };

    if pdpt_entry & PRESENT == 0 {
        return None;
    }

    let pd =
        (pdpt_entry & ADDRESS_MASK)
            as *mut PageTable;

    let pd_entry = unsafe {
        (*pd).entries[indices.pd]
    };

    if pd_entry & PRESENT == 0 {
        return None;
    }

    if pd_entry & HUGE_PAGE != 0 {
        return Some([
            pml4_entry,
            pdpt_entry,
            pd_entry,
            0,
        ]);
    }

    let pt =
        (pd_entry & ADDRESS_MASK)
            as *mut PageTable;

    let pte = unsafe {
        (*pt).entries[indices.pt]
    };

    Some([
        pml4_entry,
        pdpt_entry,
        pd_entry,
        pte,
    ])
}

pub use address_space::{
    AddressSpace,
    Mapper,
};

pub use mapper::{
    map_page,
    unmap_page,
};

pub use registers::current_pml4;