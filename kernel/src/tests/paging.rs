use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

use super::framework::TestRunner;

pub fn run(
    runner: &mut TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"paging::duplicate_mapping",
        || test_duplicate_mapping(allocator),
    );

    runner.run(
        b"paging::mapping",
        || test_mapping(allocator),
    );

    runner.run(
        b"paging::unmap",
        || test_unmap(allocator),
    );

    runner.run(
        b"paging::unmap_rejection",
        || test_unmap_rejection(allocator),
    );

    runner.run(
        b"paging::user_mapping_permissions",
        || test_user_mapping_permissions(allocator),
    );
}

fn test_duplicate_mapping(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let duplicate_virtual =
        0xFFFF_9000_0000_2000;

    let duplicate_frame_a =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let duplicate_frame_b =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    unsafe {
        if mapper
            .map(
                allocator,
                duplicate_virtual,
                duplicate_frame_a.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    if !allocator
        .is_frame_used(duplicate_frame_a)
        .unwrap_or(false)
    {
        return false;
    }

    unsafe {
        if mapper
            .map(
                allocator,
                duplicate_virtual,
                duplicate_frame_b.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: false,
                },
            )
            .is_ok()
        {
            return false;
        }
    }

    allocator
        .is_frame_used(duplicate_frame_b)
        .unwrap_or(false)
}

fn test_mapping(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mapping_virtual =
        0xFFFF_9000_0000_3000;

    let mapping_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let mapping_physical =
        mapping_frame.start_address;

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    unsafe {
        if mapper
            .map(
                allocator,
                mapping_virtual,
                mapping_physical,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    let virtual_ptr =
        mapping_virtual as *mut u64;

    unsafe {
        virtual_ptr.write(
            0xAABB_CCDD_1122_3344,
        );

        if virtual_ptr.read()
            != 0xAABB_CCDD_1122_3344
        {
            return false;
        }
    }

    let physical_ptr =
        mapping_physical as *const u64;

    unsafe {
        if physical_ptr.read()
            != 0xAABB_CCDD_1122_3344
        {
            return false;
        }
    }

    true
}

fn test_unmap(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let unmap_virtual =
        0xFFFF_9000_0000_4000;

    let unmap_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let unmap_physical =
        unmap_frame.start_address;

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    unsafe {
        if mapper
            .map(
                allocator,
                unmap_virtual,
                unmap_physical,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    let unmap_ptr =
        unmap_virtual as *mut u64;

    unsafe {
        unmap_ptr.write(
            0x5566_7788_AABB_CCDD,
        );

        if unmap_ptr.read()
            != 0x5566_7788_AABB_CCDD
        {
            return false;
        }
    }

    let unmapped_physical =
        unsafe {
            match mapper.unmap(unmap_virtual) {
                Ok(address) => address,
                Err(()) => return false,
            }
        };

    if unmapped_physical != unmap_physical {
        return false;
    }

    if !allocator
        .is_frame_used(unmap_frame)
        .unwrap_or(false)
    {
        return false;
    }

    let returned_frame =
        match memory::physical::Frame::new(
            unmapped_physical,
        ) {
            Some(frame) => frame,
            None => return false,
        };

    if allocator
        .free_frame(returned_frame)
        .is_err()
    {
        return false;
    }

    if allocator
        .is_frame_used(returned_frame)
        .unwrap_or(true)
    {
        return false;
    }

    let reused_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    if reused_frame.start_address
        != unmap_physical
    {
        return false;
    }

    let remap_physical =
        reused_frame.start_address;

    unsafe {
        let mut mapper =
            memory::paging::Mapper::new(pml4);

        if mapper
            .map(
                allocator,
                unmap_virtual,
                remap_physical,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    unsafe {
        unmap_ptr.write(
            0x1122_3344_5566_7788,
        );

        if unmap_ptr.read()
            != 0x1122_3344_5566_7788
        {
            return false;
        }
    }

    true
}

fn test_unmap_rejection(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let _ = allocator;

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    let noncanonical_virtual =
        0x0000_8000_0000_0000;

    unsafe {
        if mapper
            .unmap(noncanonical_virtual)
            .is_ok()
        {
            return false;
        }
    }

    let unaligned_virtual =
        0xFFFF_9000_0000_5001;

    unsafe {
        if mapper
            .unmap(unaligned_virtual)
            .is_ok()
        {
            return false;
        }
    }

    let unmapped_virtual =
        0xFFFF_9000_0000_6000;

    unsafe {
        if mapper
            .unmap(unmapped_virtual)
            .is_ok()
        {
            return false;
        }
    }

    let huge_page_virtual =
        0x0000_0020_0000;

    unsafe {
        if mapper
            .unmap(huge_page_virtual)
            .is_ok()
        {
            return false;
        }
    }

    true
}

fn test_user_mapping_permissions(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let user_virtual =
        0x0000_4000_0000_0000;

    let user_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    unsafe {
        if mapper
            .map(
                allocator,
                user_virtual,
                user_frame.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: true,
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
                user_virtual,
            ) {
                Some(entries) => entries,
                None => return false,
            }
        };

    let user_bit = 1u64 << 2;

    // PML4 -> PDPT -> PD -> PT/PTE
    if entries[0] & user_bit == 0 {
        return false;
    }

    if entries[1] & user_bit == 0 {
        return false;
    }

    if entries[2] & user_bit == 0 {
        return false;
    }

    if entries[3] & user_bit == 0 {
        return false;
    }

    // Kernel mapping must remain supervisor-only.
    let kernel_virtual =
        0xFFFF_9000_0000_7000;

    let kernel_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    unsafe {
        if mapper
            .map(
                allocator,
                kernel_virtual,
                kernel_frame.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: false,
                },
            )
            .is_err()
        {
            return false;
        }
    }

    let kernel_entries =
        unsafe {
            match memory::paging::test_entry(
                pml4,
                kernel_virtual,
            ) {
                Some(entries) => entries,
                None => return false,
            }
        };

    if kernel_entries[3] & user_bit != 0 {
        return false;
    }

    // USER mappings must not enter the high-half.
    let high_user_virtual =
        0xFFFF_9000_0000_8000;

    let high_user_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    unsafe {
        if mapper
            .map(
                allocator,
                high_user_virtual,
                high_user_frame.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: true,
                },
            )
            .is_ok()
        {
            return false;
        }
    }

    // USER mappings must not reuse the existing
    // supervisor-only identity-map hierarchy.
    let identity_user_virtual =
        0x0000_0000_0040_0000;

    let identity_user_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    unsafe {
        if mapper
            .map(
                allocator,
                identity_user_virtual,
                identity_user_frame.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                    user: true,
                },
            )
            .is_ok()
        {
            return false;
        }
    }

    true
}