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

    unsafe {
        load_cr3(pml4_frame.start_address);
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

unsafe fn load_cr3(address: u64) {
    unsafe {
        core::arch::asm!(
            "mov cr3, {0}",
            in(reg) address,
            options(nostack, preserves_flags)
        );
    }
}