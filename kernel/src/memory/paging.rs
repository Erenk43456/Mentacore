use super::heap::{HEAP_SIZE, HEAP_START};
use super::physical::PhysicalFrameAllocator;
use mentacore_boot_protocol::BootInfo;

pub const PAGE_SIZE: u64 = 4096;
const HUGE_PAGE_SIZE: u64 = 0x20_0000;
const ENTRY_COUNT: usize = 512;

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const USER: u64 = 1 << 2;
const PCD: u64 = 1 << 4;
const HUGE_PAGE: u64 = 1 << 7;

const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;
const INDEX_MASK: u64 = 0x1ff;
const IDENTITY_MAP_SIZE: u64 = 0x1_0000_0000;

pub const USER_SPACE_START: u64 = 0x0000_0000_0000_0000;
pub const USER_SPACE_END: u64 = 0x0000_7FFF_FFFF_FFFF;

pub const KERNEL_SPACE_START: u64 = 0xFFFF_8000_0000_0000;
pub const KERNEL_SPACE_END: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[derive(Clone, Copy)]
pub struct PageFlags {
    pub writable: bool,
    pub cache_disable: bool,
    pub user: bool,
}

const KERNEL_FLAGS: PageFlags = PageFlags {
    writable: true,
    cache_disable: false,
    user: false,
};

fn flags_to_entry(flags: PageFlags) -> u64 {
    PRESENT
        | if flags.writable { WRITABLE } else { 0 }
        | if flags.cache_disable { PCD } else { 0 }
        | if flags.user { USER } else { 0 }
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

#[derive(Clone, Copy)]
struct PageTableIndices {
    pml4: usize,
    pdpt: usize,
    pd: usize,
    pt: usize,
}

fn page_table_indices(address: u64) -> PageTableIndices {
    PageTableIndices {
        pml4: ((address >> 39) & INDEX_MASK) as usize,
        pdpt: ((address >> 30) & INDEX_MASK) as usize,
        pd: ((address >> 21) & INDEX_MASK) as usize,
        pt: ((address >> 12) & INDEX_MASK) as usize,
    }
}

fn validate_mapping(
    virtual_address: u64,
    physical_address: u64,
    flags: PageFlags,
) -> Result<(), ()> {
    if !is_canonical_address(virtual_address)
        || virtual_address & (PAGE_SIZE - 1) != 0
    {
        return Err(());
    }

    if physical_address & (PAGE_SIZE - 1) != 0
        || !is_valid_physical_address(physical_address)
    {
        return Err(());
    }

    if flags.user && !is_user_address(virtual_address) {
        return Err(());
    }

    Ok(())
}

unsafe fn allocate_table(
    allocator: &mut PhysicalFrameAllocator,
) -> Result<(u64, *mut PageTable), ()> {
    let frame = allocator.allocate_frame().ok_or(())?;
    let table = frame.start_address as *mut PageTable;

    unsafe {
        (*table).zero();
    }

    Ok((frame.start_address, table))
}

unsafe fn create_page_tables(
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

unsafe fn ensure_table(
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

        return Ok((entry & ADDRESS_MASK) as *mut PageTable);
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

unsafe fn find_page_table(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Result<(*mut PageTable, PageTableIndices), ()> {
    if !is_canonical_address(virtual_address)
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

    let pdpt = (pml4_entry & ADDRESS_MASK)
        as *mut PageTable;

    let pdpt_entry = unsafe {
        (*pdpt).entries[indices.pdpt]
    };

    if pdpt_entry & PRESENT == 0
        || pdpt_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pd = (pdpt_entry & ADDRESS_MASK)
        as *mut PageTable;

    let pd_entry = unsafe {
        (*pd).entries[indices.pd]
    };

    if pd_entry & PRESENT == 0
        || pd_entry & HUGE_PAGE != 0
    {
        return Err(());
    }

    let pt = (pd_entry & ADDRESS_MASK)
        as *mut PageTable;

    Ok((pt, indices))
}

#[cfg(feature = "kernel-tests")]
pub unsafe fn test_entry(
    pml4: *mut PageTable,
    virtual_address: u64,
) -> Option<[u64; 4]> {
    if !is_canonical_address(virtual_address) {
        return None;
    }

    let indices = page_table_indices(virtual_address);

    let pml4_entry = unsafe {
        (*pml4).entries[indices.pml4]
    };

    if pml4_entry & PRESENT == 0 {
        return None;
    }

    let pdpt = (pml4_entry & ADDRESS_MASK)
        as *mut PageTable;

    let pdpt_entry = unsafe {
        (*pdpt).entries[indices.pdpt]
    };

    if pdpt_entry & PRESENT == 0 {
        return None;
    }

    let pd = (pdpt_entry & ADDRESS_MASK)
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

    let pt = (pd_entry & ADDRESS_MASK)
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

pub unsafe fn init(
    allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) -> Result<(), ()> {
    let (pml4, pml4_address, directories) =
        unsafe { create_page_tables(allocator)? };

    let pd2 = directories[2];

    map_framebuffer(
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

    let test_virtual = 0xFFFF_9000_0000_0000;

    let test_frame =
        allocator.allocate_frame().ok_or(())?;

    unsafe {
        map_page(
            pml4,
            allocator,
            test_virtual,
            test_frame.start_address,
            KERNEL_FLAGS,
        )?;
    }

    map_heap(
        pml4,
        allocator,
        HEAP_START,
        HEAP_SIZE,
    )?;

    unsafe {
        load_cr3(pml4_address);
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
        if map_page(
            pml4,
            allocator,
            invalid_virtual,
            invalid_frame.start_address,
            KERNEL_FLAGS,
        ).is_ok() {
            return Err(());
        }
    }

    let invalid_physical =
        0x0010_0000_0000_0000;

    unsafe {
        if map_page(
            pml4,
            allocator,
            0xFFFF_9000_0000_2000,
            invalid_physical,
            KERNEL_FLAGS,
        ).is_ok() {
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
    let pointer = virtual_address as *mut u64;

    unsafe {
        pointer.write(0xDEAD_BEEF_CAFE_BABE);

        if pointer.read() != 0xDEAD_BEEF_CAFE_BABE {
            return Err(());
        }
    }

    let frame =
        allocator.allocate_frame().ok_or(())?;

    unsafe {
        map_page(
            pml4,
            allocator,
            virtual_address + PAGE_SIZE,
            frame.start_address,
            KERNEL_FLAGS,
        )?;
    }

    let pointer =
        (virtual_address + PAGE_SIZE) as *mut u64;

    unsafe {
        pointer.write(0x1122_3344_5566_7788);

        if pointer.read() != 0x1122_3344_5566_7788 {
            return Err(());
        }
    }

    Ok(())
}

fn fill_identity_directory(
    page_directory: *mut PageTable,
) {
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
    validate_mapping(
        virtual_address,
        physical_address,
        flags,
    )?;

    let indices = page_table_indices(virtual_address);
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

fn map_heap(
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
        || !is_canonical_address(virtual_start)
        || !is_canonical_address(virtual_end - 1)
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

pub unsafe fn current_pml4() -> *mut PageTable {
    let address: u64;

    unsafe {
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) address,
            options(nostack, preserves_flags)
        );
    }

    (address & ADDRESS_MASK) as *mut PageTable
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
        unsafe { Mapper::new(self.pml4) }
    }

    pub unsafe fn map(
        &self,
        allocator: &mut PhysicalFrameAllocator,
        virtual_address: u64,
        physical_address: u64,
        flags: PageFlags,
    ) -> Result<(), ()> {
        let mut mapper =
            unsafe { Mapper::new(self.pml4) };

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
        let mapper =
            unsafe { Mapper::new(self.pml4) };

        unsafe { mapper.unmap(virtual_address) }
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