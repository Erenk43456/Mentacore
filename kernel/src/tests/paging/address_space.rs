use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

pub(super) fn test_address_space_abstraction(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let pml4_address =
        unsafe {
            memory::paging::current_pml4_address()
        };

    let address_space =
        unsafe {
            memory::paging::AddressSpace::from_pml4(
                pml4,
                pml4_address,
            )
        };

    if address_space.pml4() != pml4 {
        return false;
    }

    let new_address_space =
        unsafe {
            match memory::paging::AddressSpace::new(allocator) {
                Ok(address_space) => address_space,
                Err(()) => return false,
            }
        };

    let new_pml4 = new_address_space.pml4();

    if new_pml4.is_null() {
        return false;
    }

    if unsafe {
        !memory::paging::test_page_table_empty(new_pml4)
    } {
        return false;
    }

    true
}

pub(super) fn test_address_space_mapping(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let virtual_address =
        0x0000_6000_0000_0000;

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let pml4_address =
        unsafe {
            memory::paging::current_pml4_address()
        };

    let address_space =
        unsafe {
            memory::paging::AddressSpace::from_pml4(
                pml4,
                pml4_address,
            )
        };

    let frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    unsafe {
        if address_space
            .map(
                allocator,
                virtual_address,
                frame.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: true,
                    executable: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    let entries =
        unsafe {
            match memory::paging::test_entry(
                pml4,
                virtual_address,
            ) {
                Some(entries) => entries,
                None => return false,
            }
        };

    let user_bit = 1u64 << 2;

    if entries[3] & 1 == 0 {
        return false;
    }

    if entries[3] & user_bit == 0 {
        return false;
    }

    true
}

pub(super) fn test_address_space_unmapping(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let virtual_address =
        0x0000_6000_0000_1000;

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let pml4_address =
        unsafe {
            memory::paging::current_pml4_address()
        };

    let address_space =
        unsafe {
            memory::paging::AddressSpace::from_pml4(
                pml4,
                pml4_address,
            )
        };

    let frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let physical_address =
        frame.start_address;

    unsafe {
        if address_space
            .map(
                allocator,
                virtual_address,
                physical_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: true,
                    executable: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    let mapped =
        unsafe {
            memory::paging::test_entry(
                pml4,
                virtual_address,
            )
        };

    let mapped =
        match mapped {
            Some(entries) => entries,
            None => return false,
        };

    if mapped[3] & 1 == 0 {
        return false;
    }

    let unmapped =
        unsafe {
            address_space.unmap(virtual_address)
        };

    if unmapped != Ok(physical_address) {
        return false;
    }

    let after_unmap =
        unsafe {
            memory::paging::test_entry(
                pml4,
                virtual_address,
            )
        };

    match after_unmap {
        Some(entries) => entries[3] & 1 == 0,
        None => true,
    }
}

pub(super) fn test_address_space_drop() -> bool {
    let before = {
        let guard =
            crate::memory::physical::frame_allocator().lock();

        match guard.as_ref() {
            Some(allocator) =>
                allocator.allocated_count(),
            None => return false,
        }
    };

    let address_space = {
        let mut guard =
            crate::memory::physical::frame_allocator().lock();

        let allocator =
            match guard.as_mut() {
                Some(allocator) => allocator,
                None => return false,
            };

        unsafe {
            match memory::paging::AddressSpace::new_user(
                allocator,
            ) {
                Ok(address_space) => address_space,
                Err(()) => return false,
            }
        }
    };

    let mapped = {
        let mut guard =
            crate::memory::physical::frame_allocator().lock();

        let allocator =
            match guard.as_mut() {
                Some(allocator) => allocator,
                None => return false,
            };

        let frame =
            match allocator.allocate_frame() {
                Some(frame) => frame,
                None => return false,
            };

        let virtual_address =
            0x0000_7000_0000_0000;

        let result = unsafe {
            address_space.map_owned(
                allocator,
                virtual_address,
                frame.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: true,
                    executable: false,
                },
            )
        };

        result.is_ok()
    };

    if !mapped {
        drop(address_space);
        return false;
    }

    drop(address_space);

    let after = {
        let guard =
            crate::memory::physical::frame_allocator().lock();

        match guard.as_ref() {
            Some(allocator) =>
                allocator.allocated_count(),
            None => return false,
        }
    };

    after == before
}