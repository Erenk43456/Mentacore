use super::physical::PhysicalFrameAllocator;
use super::heap::{HEAP_SIZE, HEAP_START};
use mentacore_boot_protocol::BootInfo;

pub const PAGE_SIZE: u64 = 4096;
const HUGE_PAGE_SIZE: u64 = 0x20_0000;

const ENTRY_COUNT: usize = 512;

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const PCD: u64 = 1 << 4;
const HUGE_PAGE: u64 = 1 << 7;
const USER: u64 = 1 << 2;

const IDENTITY_MAP_SIZE: u64 = 0x1_0000_0000;

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

fn flags_to_entry(flags: PageFlags) -> u64 {
    let mut entry = PRESENT;

    if flags.writable {
        entry |= WRITABLE;
    }

    if flags.cache_disable {
        entry |= PCD;
    }

    if flags.user {
        entry |= USER;
    }

    entry
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [u64; ENTRY_COUNT],
}

impl PageTable {
    fn zero(&mut self) {
        for entry in &mut self.entries {
            *entry = 0;
        }
    }
}

#[cfg(feature = "kernel-tests")]
pub unsafe fn test_entry(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Option<[u64; 4]> {
    if !is_canonical_address(virtual_address) {
        return None;
    }

    let pml4_index =
        ((virtual_address >> 39) & 0x1ff) as usize;

    let pdpt_index =
        ((virtual_address >> 30) & 0x1ff) as usize;

    let pd_index =
        ((virtual_address >> 21) & 0x1ff) as usize;

    let pt_index =
        ((virtual_address >> 12) & 0x1ff) as usize;

    let pml4_entry = (*pml4).entries[pml4_index];

    if pml4_entry & PRESENT == 0 {
        return None;
    }

    let pdpt =
        (pml4_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable;

    let pdpt_entry = (*pdpt).entries[pdpt_index];

    if pdpt_entry & PRESENT == 0 {
        return None;
    }

    let pd =
        (pdpt_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable;

    let pd_entry = (*pd).entries[pd_index];

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
        (pd_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable;

    let pte = (*pt).entries[pt_index];

    Some([
        pml4_entry,
        pdpt_entry,
        pd_entry,
        pte,
    ])
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

    let invalid_virtual = 0x0000_8000_0000_0000;

    let invalid_frame =
        allocator.allocate_frame().ok_or(())?;

    unsafe {
        if map_page(
            pml4,
            allocator,
            invalid_virtual,
            invalid_frame.start_address,
            PageFlags {
                writable: true,
                cache_disable: false,
                user: false,
            },
        ).is_ok() {
            return Err(());
        }
    }

    let invalid_physical = 0x0010_0000_0000_0000;

    unsafe {
        if map_page(
            pml4,
            allocator,
            0xFFFF_9000_0000_2000,
            invalid_physical,
            PageFlags {
                writable: true,
                cache_disable: false,
                user: false,
            },
        ).is_ok() {
            return Err(());
        }
    }

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
                user: false,
            },
        )?;
    }

    map_heap(
        pml4,
        allocator,
        HEAP_START,
        HEAP_SIZE,
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
                user: false,
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

fn is_canonical_address(address: u64) -> bool {
    let sign_bit = (address >> 47) & 1;
    let upper_bits = address >> 48;

    if sign_bit == 0 {
        upper_bits == 0
    } else {
        upper_bits == 0xFFFF
    }
}

fn is_valid_physical_address(address: u64) -> bool {
    (address >> 52) == 0
}

pub fn is_user_address(address: u64) -> bool {
    address >= USER_SPACE_START
        && address <= USER_SPACE_END
}

pub fn is_kernel_address(address: u64) -> bool {
    address >= KERNEL_SPACE_START
        && address <= KERNEL_SPACE_END
}

pub unsafe fn map_page(
    pml4: *mut PageTable,
    allocator: &mut PhysicalFrameAllocator,
    virtual_address: u64,
    physical_address: u64,
    flags: PageFlags,
) -> Result<(), ()> {
    if !is_canonical_address(virtual_address) {
        return Err(());
    }

    if virtual_address & (PAGE_SIZE - 1) != 0 {
        return Err(());
    }

    if physical_address & (PAGE_SIZE - 1) != 0 {
        return Err(());
    }

    if !is_valid_physical_address(physical_address) {
        return Err(());
    }

    if flags.user && !is_user_address(virtual_address) {
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
        let entry = unsafe {
            (*pml4).entries[pml4_index]
        };

        if flags.user && entry & USER == 0 {
            return Err(());
        }

        (entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable
    } else {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        let table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*table).zero();

            let mut entry =
                frame.start_address
                | PRESENT
                | WRITABLE;

            if flags.user {
                entry |= USER;
            }

            (*pml4).entries[pml4_index] = entry;
        }

        table
    };

    // PDPT -> PD
    let pd = if unsafe {
        (*pdpt).entries[pdpt_index] & PRESENT
    } != 0 {
        let entry = unsafe {
            (*pdpt).entries[pdpt_index]
        };

        if flags.user && entry & USER == 0 {
            return Err(());
        }

        (entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable
    } else {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        let table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*table).zero();

            let mut entry =
                frame.start_address
                | PRESENT
                | WRITABLE;

            if flags.user {
                entry |= USER;
            }

            (*pdpt).entries[pdpt_index] = entry;
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

        if flags.user && pd_entry & USER == 0 {
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

            let mut entry =
                frame.start_address
                | PRESENT
                | WRITABLE;

            if flags.user {
                entry |= USER;
            }

            (*pd).entries[pd_index] = entry;
        }

        table
    };

    // PT -> physical frame
    unsafe {
        if (*pt).entries[pt_index] & PRESENT != 0 {
            return Err(());
        }

        (*pt).entries[pt_index] =
            physical_address
            | flags_to_entry(flags);
    }

    Ok(())
}

pub unsafe fn unmap_page(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Result<u64, ()> {
    if !is_canonical_address(virtual_address) {
        return Err(());
    }

    if virtual_address & (PAGE_SIZE - 1) != 0 {
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
    let pml4_entry = unsafe {
        (*pml4).entries[pml4_index]
    };

    if pml4_entry & PRESENT == 0 {
        return Err(());
    }

    let pdpt =
        (pml4_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable;

    // PDPT -> PD
    let pdpt_entry = unsafe {
        (*pdpt).entries[pdpt_index]
    };

    if pdpt_entry & PRESENT == 0 {
        return Err(());
    }

    if pdpt_entry & HUGE_PAGE != 0 {
        return Err(());
    }

    let pd =
        (pdpt_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable;

    // PD -> PT
    let pd_entry = unsafe {
        (*pd).entries[pd_index]
    };

    if pd_entry & PRESENT == 0 {
        return Err(());
    }

    if pd_entry & HUGE_PAGE != 0 {
        return Err(());
    }

    let pt =
        (pd_entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable;

    // PT -> physical frame
    let pte = unsafe {
        (*pt).entries[pt_index]
    };

    if pte & PRESENT == 0 {
        return Err(());
    }

    let physical_address =
        pte & 0x000f_ffff_ffff_f000;

    // Remove mapping.
    unsafe {
        (*pt).entries[pt_index] = 0;
    }

    // Remove stale TLB entry.
    unsafe {
        core::arch::asm!(
            "invlpg [{}]",
            in(reg) virtual_address,
            options(nostack, preserves_flags)
        );
    }

    Ok(physical_address)
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

    if virtual_start & (PAGE_SIZE - 1) != 0 {
        return Err(());
    }

    let virtual_end =
        virtual_start.checked_add(size).ok_or(())?;

    if virtual_end <= virtual_start {
        return Err(());
    }

    if !is_canonical_address(virtual_start)
        || !is_canonical_address(virtual_end - 1)
    {
        return Err(());
    }

    let pml4_index =
        ((virtual_start >> 39) & 0x1ff) as usize;

    let first_pdpt_index =
        ((virtual_start >> 30) & 0x1ff) as usize;

    let last_pdpt_index =
        (((virtual_end - 1) >> 30) & 0x1ff) as usize;

    // The heap must remain inside one PDPT entry.
    if first_pdpt_index != last_pdpt_index {
        return Err(());
    }

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
        (*pdpt).entries[first_pdpt_index] & PRESENT
    } != 0 {
        let entry = unsafe {
            (*pdpt).entries[first_pdpt_index]
        };

        if entry & HUGE_PAGE != 0 {
            return Err(());
        }

        (entry & 0x000f_ffff_ffff_f000)
            as *mut PageTable
    } else {
        let frame =
            allocator.allocate_frame().ok_or(())?;

        let table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*table).zero();

            (*pdpt).entries[first_pdpt_index] =
                frame.start_address
                | PRESENT
                | WRITABLE;
        }

        table
    };

    let first_pd_index =
        ((virtual_start >> 21) & 0x1ff) as usize;

    let last_pd_index =
        (((virtual_end - 1) >> 21) & 0x1ff) as usize;

    // Reserve the page tables covering the heap.
    //
    // The PTs themselves are present, but their PTEs
    // remain non-present. Physical heap frames will be
    // allocated later by the page-fault handler.
    for pd_index in first_pd_index..=last_pd_index {
        let pd_entry = unsafe {
            (*pd).entries[pd_index]
        };

        if pd_entry & PRESENT != 0 {
            if pd_entry & HUGE_PAGE != 0 {
                return Err(());
            }

            continue;
        }

        let frame =
            allocator.allocate_frame().ok_or(())?;

        let page_table =
            frame.start_address as *mut PageTable;

        unsafe {
            (*page_table).zero();

            (*pd).entries[pd_index] =
                frame.start_address
                | PRESENT
                | WRITABLE;
        }
    }

    Ok(())
}

pub unsafe fn current_pml4() -> *mut PageTable {
    let address: u64;

    unsafe {
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) address,
            options(nostack, preserves_flags)
        );
    }

    (address & 0x000f_ffff_ffff_f000) as *mut PageTable
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

pub struct AddressSpace {
    pml4: *mut PageTable,
}

impl AddressSpace {
    pub unsafe fn from_pml4(
        pml4: *mut PageTable,
    ) -> Self {
        Self { pml4 }
    }

    pub fn pml4(&self) -> *mut PageTable {
        self.pml4
    }

    pub unsafe fn mapper(&self) -> Mapper {
        unsafe {
            Mapper::new(self.pml4)
        }
    }

    pub unsafe fn map(
        &self,
        allocator: &mut PhysicalFrameAllocator,
        virtual_address: u64,
        physical_address: u64,
        flags: PageFlags,
    ) -> Result<(), ()> {
        let mut mapper = unsafe {
            Mapper::new(self.pml4)
        };

        unsafe {
            mapper.map(
                allocator,
                virtual_address,
                physical_address,
                flags,
            )
        }
    }

    pub unsafe fn unmap(
        &self,
        virtual_address: u64,
    ) -> Result<u64, ()> {
        let mapper = unsafe {
            Mapper::new(self.pml4)
        };

        unsafe {
            mapper.unmap(virtual_address)
        }
    }
}

pub struct Mapper {
    pml4: *mut PageTable,
}

impl Mapper {
    pub unsafe fn new(
        pml4: *mut PageTable,
    ) -> Self {
        Self { pml4 }
    }

    pub unsafe fn map(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
        virtual_address: u64,
        physical_address: u64,
        flags: PageFlags,
    ) -> Result<(), ()> {
        unsafe {
            map_page(
                self.pml4,
                allocator,
                virtual_address,
                physical_address,
                flags,
            )
        }
    }

    pub unsafe fn unmap(
        &self,
        virtual_address: u64,
    ) -> Result<u64, ()> {
        unsafe {
            unmap_page(
                self.pml4,
                virtual_address,
            )
        }
    }
}