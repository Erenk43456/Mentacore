use super::physical::PhysicalFrameAllocator;
use mentacore_boot_protocol::BootInfo;

const PAGE_SIZE: u64 = 4096;
const HUGE_PAGE_SIZE: u64 = 0x20_0000;

const ENTRY_COUNT: usize = 512;

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const PCD: u64 = 1 << 4;
const HUGE_PAGE: u64 = 1 << 7;

const IDENTITY_MAP_SIZE: u64 = 0x1_0000_0000;

#[derive(Clone, Copy)]
pub struct PageFlags {
    pub writable: bool,
    pub cache_disable: bool,
}

fn flags_to_entry(flags: PageFlags) -> u64 {
    let mut entry = PRESENT;

    if flags.writable {
        entry |= WRITABLE;
    }

    if flags.cache_disable {
        entry |= PCD;
    }

    entry
}

#[repr(align(4096))]
struct PageTable {
    entries: [u64; ENTRY_COUNT],
}

impl PageTable {
    fn zero(&mut self) {
        for entry in &mut self.entries {
            *entry = 0;
        }
    }
}

pub unsafe fn init(
    allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) -> Result<(), ()> {
    let pml4_frame = allocator.allocate_frame().ok_or(())?;
    let pdpt_frame = allocator.allocate_frame().ok_or(())?;

    let pd0_frame = allocator.allocate_frame().ok_or(())?;
    let pd1_frame = allocator.allocate_frame().ok_or(())?;
    let pd2_frame = allocator.allocate_frame().ok_or(())?;
    let pd3_frame = allocator.allocate_frame().ok_or(())?;

    let pml4 = pml4_frame.start_address as *mut PageTable;
    let pdpt = pdpt_frame.start_address as *mut PageTable;

    let pd0 = pd0_frame.start_address as *mut PageTable;
    let pd1 = pd1_frame.start_address as *mut PageTable;
    let pd2 = pd2_frame.start_address as *mut PageTable;
    let pd3 = pd3_frame.start_address as *mut PageTable;

    unsafe {
        (*pml4).zero();
        (*pdpt).zero();

        (*pd0).zero();
        (*pd1).zero();
        (*pd2).zero();
        (*pd3).zero();
    }

    unsafe {
        (*pml4).entries[0] =
            pdpt_frame.start_address | PRESENT | WRITABLE;

        (*pdpt).entries[0] =
            pd0_frame.start_address | PRESENT | WRITABLE;

        (*pdpt).entries[1] =
            pd1_frame.start_address | PRESENT | WRITABLE;

        (*pdpt).entries[2] =
            pd2_frame.start_address | PRESENT | WRITABLE;

        (*pdpt).entries[3] =
            pd3_frame.start_address | PRESENT | WRITABLE;
    }

    fill_identity_directory(pd0);
    fill_identity_directory(pd1);
    fill_identity_directory(pd2);
    fill_identity_directory(pd3);

    map_framebuffer(
        pd2,
        allocator,
        boot_info,
    )?;

    let test_virtual = 0xFFFF_9000_0000_0000;

    let test_frame =
        allocator.allocate_frame().ok_or(())?;

    let test_physical =
        test_frame.start_address;

    unsafe {
        map_page(
            pml4,
            allocator,
            test_virtual,
            test_physical,
            PageFlags {
                writable: true,
                cache_disable: false,
            },
        )?;
    }

    map_heap(
        pml4,
        allocator,
        0xFFFF_8000_0000_0000,
        1024 * 1024,
    )?;

    unsafe {
        load_cr3(pml4_frame.start_address);
    }

    let test_ptr =
        test_virtual as *mut u64;

    unsafe {
        test_ptr.write(0xDEAD_BEEF_CAFE_BABE);

        if test_ptr.read() != 0xDEAD_BEEF_CAFE_BABE {
            return Err(());
        }
    }

    let test_frame_2 =
        allocator.allocate_frame().ok_or(())?;

    let test_physical_2 =
        test_frame_2.start_address;

    unsafe {
        map_page(
            pml4,
            allocator,
            test_virtual + PAGE_SIZE,
            test_physical_2,
            PageFlags {
                writable: true,
                cache_disable: false,
            },
        )?;
    }

    let test_ptr_2 =
        (test_virtual + PAGE_SIZE) as *mut u64;

    unsafe {
        test_ptr_2.write(0x1122_3344_5566_7788);

        if test_ptr_2.read() != 0x1122_3344_5566_7788 {
            return Err(());
        }
    }

    Ok(())
}

fn fill_identity_directory(page_directory: *mut PageTable) {
    for index in 0..ENTRY_COUNT {
        let physical_address =
            (index as u64) * HUGE_PAGE_SIZE;

        unsafe {
            (*page_directory).entries[index] =
                physical_address
                | PRESENT
                | WRITABLE
                | HUGE_PAGE;
        }
    }
}

fn map_framebuffer(
    pd2: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) -> Result<(), ()> {
    let framebuffer_start = boot_info.framebuffer_addr;
    let framebuffer_size = boot_info.framebuffer_size;

    if framebuffer_start == 0 || framebuffer_size == 0 {
        return Err(());
    }

    let framebuffer_end = framebuffer_start
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

    for huge_page_index in first_huge_page..=last_huge_page {
        let pd_index =
            (huge_page_index % ENTRY_COUNT as u64) as usize;

        let page_table_frame =
            allocator.allocate_frame().ok_or(())?;

        let page_table =
            page_table_frame.start_address as *mut PageTable;

        unsafe {
            (*page_table).zero();

            (*pd2).entries[pd_index] =
                page_table_frame.start_address
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

unsafe fn map_page(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    virtual_address: u64,
    physical_address: u64,
    flags: PageFlags,
) -> Result<(), ()> {
    if virtual_address & (PAGE_SIZE - 1) != 0 {
        return Err(());
    }

    if physical_address & (PAGE_SIZE - 1) != 0 {
        return Err(());
    }

    let pml4_index =
        ((virtual_address >> 39) & 0x1ff) as usize;

    let pdpt_index =
        ((virtual_address >> 30) & 0x1ff) as usize;

    let pd_index =
        ((virtual_address >> 21) & 0x1ff) as usize;

    let pt_index =
        ((virtual_address >> 12) & 0x1ff) as usize;

    // PML4 -> PDPT
    let pdpt = if unsafe {
        (*pml4).entries[pml4_index] & PRESENT
    } != 0 {
        (unsafe {
            (*pml4).entries[pml4_index]
        } & 0x000f_ffff_ffff_f000) as *mut PageTable
    } else {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        let table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*table).zero();

            (*pml4).entries[pml4_index] =
                frame.start_address
                | PRESENT
                | WRITABLE;
        }

        table
    };

    // PDPT -> PD
    let pd = if unsafe {
        (*pdpt).entries[pdpt_index] & PRESENT
    } != 0 {
        (unsafe {
            (*pdpt).entries[pdpt_index]
        } & 0x000f_ffff_ffff_f000) as *mut PageTable
    } else {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        let table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*table).zero();

            (*pdpt).entries[pdpt_index] =
                frame.start_address
                | PRESENT
                | WRITABLE;
        }

        table
    };

    // PD -> PT
    let pd_entry = unsafe {
        (*pd).entries[pd_index]
    };

    let pt = if pd_entry & PRESENT != 0 {
        if pd_entry & HUGE_PAGE != 0 {
            return Err(());
        }

        (pd_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable
    } else {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        let table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*table).zero();

            (*pd).entries[pd_index] =
                frame.start_address
                | PRESENT
                | WRITABLE;
        }

        table
    };

    // PT -> physical frame
    unsafe {
        (*pt).entries[pt_index] =
            physical_address
            | flags_to_entry(flags);
    }

    Ok(())
}

fn map_heap(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    virtual_start: u64,
    size: u64,
) -> Result<(), ()> {
    if size == 0 {
        return Err(());
    }

    let page_count =
        (size + PAGE_SIZE - 1) / PAGE_SIZE;

    let pml4_index =
        ((virtual_start >> 39) & 0x1ff) as usize;

    let pdpt_frame =
        allocator.allocate_frame().ok_or(())?;

    let pdpt =
        pdpt_frame.start_address as *mut PageTable;

    unsafe {
        (*pdpt).zero();

        (*pml4).entries[pml4_index] =
            pdpt_frame.start_address
            | PRESENT
            | WRITABLE;
    }

    let pdpt_index =
        ((virtual_start >> 30) & 0x1ff) as usize;

    let pd_frame =
        allocator.allocate_frame().ok_or(())?;

    let pd =
        pd_frame.start_address as *mut PageTable;

    unsafe {
        (*pd).zero();

        (*pdpt).entries[pdpt_index] =
            pd_frame.start_address
            | PRESENT
            | WRITABLE;
    }

    let pd_index =
        ((virtual_start >> 21) & 0x1ff) as usize;

    let pt_frame =
        allocator.allocate_frame().ok_or(())?;

    let pt =
        pt_frame.start_address as *mut PageTable;

    unsafe {
        (*pt).zero();

        (*pd).entries[pd_index] =
            pt_frame.start_address
            | PRESENT
            | WRITABLE;
    }

    for page_index in 0..page_count {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        unsafe {
            (*pt).entries[page_index as usize] =
                frame.start_address
                | PRESENT
                | WRITABLE;
        }
    }

    Ok(())
}

unsafe fn load_cr3(address: u64) {
    unsafe {
        core::arch::asm!(
            "mov cr3, {0}",
            in(reg) address,
            options(nostack, preserves_flags)
        );
    }
}