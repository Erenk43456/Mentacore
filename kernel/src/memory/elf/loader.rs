use crate::memory::paging::{
    AddressSpace,
    PageFlags,
    IDENTITY_MAP_SIZE,
    PAGE_SIZE,
};

use crate::memory::physical::PhysicalFrameAllocator;

use super::{
    ElfError,
    LoadSegment,
    ParsedElf,
};

pub const MAX_LOADED_SEGMENTS: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElfLoadError {
    Parse(ElfError),
    AddressSpaceCreation,
    AllocationFailed,
    MappingFailed,
    AddressOverflow,
    PhysicalAddressOutsideIdentityMap,
}

#[derive(Clone, Copy)]
pub struct LoadedSegment {
    pub virtual_address: u64,
    pub physical_address: u64,
    pub page_count: u64,
    pub file_size: u64,
    pub memory_size: u64,
}

impl LoadedSegment {
    pub const EMPTY: Self = Self {
        virtual_address: 0,
        physical_address: 0,
        page_count: 0,
        file_size: 0,
        memory_size: 0,
    };
}

pub const USER_STACK_PAGES: usize = 4;

pub const USER_STACK_TOP: u64 =
    crate::memory::paging::USER_SPACE_END & !(crate::memory::paging::PAGE_SIZE - 1);

pub const USER_STACK_BASE: u64 =
    USER_STACK_TOP -
    (USER_STACK_PAGES as u64 * crate::memory::paging::PAGE_SIZE);

pub struct LoadedElf {
    address_space: AddressSpace,
    entry: u64,
    segments: [LoadedSegment; MAX_LOADED_SEGMENTS],
    segment_count: usize,
    user_stack_top: u64,
}

impl LoadedElf {
    pub fn entry(&self) -> u64 {
        self.entry
    }

    pub fn segment_count(&self) -> usize {
        self.segment_count
    }

    pub fn user_stack_top(&self) -> u64 {
        self.user_stack_top
    }

    pub fn segment(
        &self,
        index: usize,
    ) -> Option<&LoadedSegment> {
        if index >= self.segment_count {
            None
        } else {
            Some(&self.segments[index])
        }
    }

    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
    }

    pub fn into_address_space(self) -> AddressSpace {
        self.address_space
    }

    pub unsafe fn map_user_stack(
        &mut self,
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<u64, ()> {
        use crate::memory::paging::{
            PageFlags,
            PAGE_SIZE,
        };

        for page_index in 0..USER_STACK_PAGES {
            let virtual_address =
                USER_STACK_BASE
                    + page_index as u64 * PAGE_SIZE;

            let frame =
                allocator.allocate_frame()
                    .ok_or(())?;

            unsafe {
                core::ptr::write_bytes(
                    frame.start_address as *mut u8,
                    0,
                    PAGE_SIZE as usize,
                );

                self.address_space.map(
                    allocator,
                    virtual_address,
                    frame.start_address,
                    PageFlags {
                        writable: true,
                        cache_disable: false,
                        user: true,
                    },
                )?;
            }
        }

        self.user_stack_top = USER_STACK_TOP;

        Ok(USER_STACK_TOP)
    }
}

pub unsafe fn load(
    data: &[u8],
    allocator: &mut PhysicalFrameAllocator,
) -> Result<LoadedElf, ElfLoadError> {
    let elf =
        ParsedElf::parse(data)
            .map_err(ElfLoadError::Parse)?;

    let address_space =
        unsafe {
            AddressSpace::new_user(
                allocator,
            )
        }
        .map_err(
            |_| ElfLoadError::AddressSpaceCreation,
        )?;

    let mut segments =
        [LoadedSegment::EMPTY;
            MAX_LOADED_SEGMENTS];

    let mut segment_count = 0usize;

    for index in 0..elf.segment_count() {
        let segment =
            elf.segment(index)
                .ok_or(
                    ElfLoadError::AddressOverflow,
                )?;

        let loaded =
            unsafe {
                load_segment(
                    &address_space,
                    allocator,
                    &elf,
                    segment,
                )
            }?;

        segments[segment_count] =
            loaded;

        segment_count += 1;
    }

    Ok(LoadedElf {
        address_space,
        entry: elf.entry(),
        segments,
        segment_count,
        user_stack_top: 0,
    })
}

unsafe fn load_segment(
    address_space: &AddressSpace,
    allocator: &mut PhysicalFrameAllocator,
    elf: &ParsedElf<'_>,
    segment: &LoadSegment,
) -> Result<LoadedSegment, ElfLoadError> {
    if segment.memory_size == 0 {
        return Err(
            ElfLoadError::AddressOverflow,
        );
    }

    let segment_start =
        segment.virtual_address;

    let segment_end =
        segment_start
            .checked_add(segment.memory_size)
            .ok_or(
                ElfLoadError::AddressOverflow,
            )?;

    let page_start =
        segment_start
            & !(PAGE_SIZE - 1);

    let page_end =
        align_up(segment_end)?;

    let page_count =
        (page_end - page_start)
            / PAGE_SIZE;

    let segment_data =
        elf.segment_data(segment)
            .map_err(ElfLoadError::Parse)?;

    let flags = PageFlags {
        writable: segment.flags.writable,
        cache_disable: false,
        user: true,
    };

    let mut first_physical_address = 0u64;

    for page_index in 0..page_count {
        let page_address =
            page_start
                .checked_add(
                    page_index
                        .checked_mul(PAGE_SIZE)
                        .ok_or(
                            ElfLoadError::AddressOverflow,
                        )?,
                )
                .ok_or(
                    ElfLoadError::AddressOverflow,
                )?;

        let frame =
            allocator
                .allocate_frame()
                .ok_or(
                    ElfLoadError::AllocationFailed,
                )?;

        let physical_address =
            frame.start_address;

        if physical_address
            >= IDENTITY_MAP_SIZE
        {
            return Err(
                ElfLoadError::
                    PhysicalAddressOutsideIdentityMap,
            );
        }

        if page_index == 0 {
            first_physical_address =
                physical_address;
        }

        unsafe {
            address_space
                .map(
                    allocator,
                    page_address,
                    physical_address,
                    flags,
                )
                .map_err(
                    |_| ElfLoadError::MappingFailed,
                )?;
        }

        unsafe {
            zero_page(
                physical_address,
            );
        }

        unsafe {
            copy_segment_page(
                physical_address,
                page_address,
                segment_start,
                segment.file_size,
                segment_data,
            )?;
        }
    }

    Ok(LoadedSegment {
        virtual_address: page_start,
        physical_address:
            first_physical_address,
        page_count,
        file_size: segment.file_size,
        memory_size: segment.memory_size,
    })
}

fn align_up(
    address: u64,
) -> Result<u64, ElfLoadError> {
    let adjusted =
        address
            .checked_add(
                PAGE_SIZE - 1,
            )
            .ok_or(
                ElfLoadError::AddressOverflow,
            )?;

    Ok(
        adjusted
            & !(PAGE_SIZE - 1),
    )
}

unsafe fn zero_page(
    physical_address: u64,
) {
    let ptr =
        physical_address
            as *mut u8;

    for index in 0..PAGE_SIZE as usize {
        unsafe {
            ptr
                .add(index)
                .write(0);
        }
    }
}

unsafe fn copy_segment_page(
    physical_address: u64,
    page_address: u64,
    segment_start: u64,
    file_size: u64,
    segment_data: &[u8],
) -> Result<(), ElfLoadError> {
    let page_end =
        page_address
            .checked_add(PAGE_SIZE)
            .ok_or(
                ElfLoadError::AddressOverflow,
            )?;

    let file_end =
        segment_start
            .checked_add(file_size)
            .ok_or(
                ElfLoadError::AddressOverflow,
            )?;

    let copy_start =
        if page_address > segment_start {
            page_address
        } else {
            segment_start
        };

    let copy_end =
        if page_end < file_end {
            page_end
        } else {
            file_end
        };

    if copy_start >= copy_end {
        return Ok(());
    }

    let source_offset =
        usize::try_from(
            copy_start
                .checked_sub(segment_start)
                .ok_or(
                    ElfLoadError::AddressOverflow,
                )?,
        )
        .map_err(
            |_| ElfLoadError::AddressOverflow,
        )?;

    let destination_offset =
        usize::try_from(
            copy_start
                .checked_sub(page_address)
                .ok_or(
                    ElfLoadError::AddressOverflow,
                )?,
        )
        .map_err(
            |_| ElfLoadError::AddressOverflow,
        )?;

    let length =
        usize::try_from(
            copy_end
                .checked_sub(copy_start)
                .ok_or(
                    ElfLoadError::AddressOverflow,
                )?,
        )
        .map_err(
            |_| ElfLoadError::AddressOverflow,
        )?;

    let source_end =
        source_offset
            .checked_add(length)
            .ok_or(
                ElfLoadError::AddressOverflow,
            )?;

    if source_end > segment_data.len() {
        return Err(
            ElfLoadError::AddressOverflow,
        );
    }

    let destination =
        (physical_address as *mut u8)
            .add(destination_offset);

    for index in 0..length {
        unsafe {
            destination
                .add(index)
                .write(
                    segment_data[
                        source_offset + index
                    ],
                );
        }
    }

    Ok(())
}