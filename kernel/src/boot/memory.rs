use mentacore_boot_protocol::BootInfo;

use crate::debug;
use crate::memory;
use crate::memory::memory_map::MemoryMap;
use crate::memory::physical::PhysicalFrameAllocator;

pub unsafe fn initialize(
    boot_info: &BootInfo,
) -> PhysicalFrameAllocator {
    let memory_map =
        match unsafe {
            MemoryMap::from_boot_info(boot_info)
        } {
            Some(map) => map,

            None => {
                debug::write(
                    b"ERROR: Invalid memory map\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let highest_conventional_address =
        match memory_map.highest_conventional_address() {
            Some(address) => address,

            None => {
                debug::write(
                    b"ERROR: No conventional memory found.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let frame_count =
        match memory::physical::frame_count_for_address(
            highest_conventional_address,
        ) {
            Some(count) => count,

            None => {
                debug::write(
                    b"ERROR: Failed to calculate frame count.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let _bitmap_size =
        match memory::physical::bitmap_size_bytes(
            frame_count,
        ) {
            Some(size) => size,

            None => {
                debug::write(
                    b"ERROR: Failed to calculate bitmap size.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let bitmap_pages =
        match memory::physical::bitmap_page_count(
            frame_count,
        ) {
            Some(pages) => pages,

            None => {
                debug::write(
                    b"ERROR: Failed to calculate bitmap page count.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let bitmap_address =
        match memory_map.find_conventional_region(
            bitmap_pages * memory::paging::PAGE_SIZE,
        ) {
            Some(address) => address,

            None => {
                debug::write(
                    b"ERROR: Failed to find bitmap region.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let mut frame_bitmap = match unsafe {
        memory::physical::FrameBitmap::new(
            bitmap_address,
            frame_count,
        )
    } {
        Some(bitmap) => bitmap,

        None => {
            debug::write(
                b"ERROR: Failed to create frame bitmap.\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    };

    unsafe {
        frame_bitmap.clear_all();
    }

    // Start with every physical frame marked as used.
    for frame in 0..frame_count {
        unsafe {
            frame_bitmap.set(frame);
        }
    }

    // UEFI Conventional Memory is available.
    for index in 0..memory_map.descriptor_count() {
        let descriptor =
            match unsafe {
                memory_map.descriptor(index)
            } {
                Some(descriptor) => descriptor,

                None => {
                    debug::write(
                        b"ERROR: Failed to read descriptor for bitmap\r\n"
                    );

                    loop {
                        core::hint::spin_loop();
                    }
                }
            };

        if descriptor.ty == 7 {
            unsafe {
                frame_bitmap.mark_free_range(
                    descriptor.physical_start,
                    descriptor.number_of_pages,
                );
            }
        }
    }

    // The bitmap's own physical pages must remain reserved.
    let bitmap_frame =
        bitmap_address / memory::paging::PAGE_SIZE;

    for frame in 0..bitmap_pages {
        unsafe {
            frame_bitmap.set(
                bitmap_frame + frame
            );
        }
    }

    // Physical frame 0 is permanently reserved.
    unsafe {
        frame_bitmap.set(0);
    }

    if !unsafe { frame_bitmap.is_used(0) } {
        debug::write(
            b"ERROR: Frame 0 is not reserved\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    let bitmap_first_frame =
        bitmap_address / memory::paging::PAGE_SIZE;

    if !unsafe {
        frame_bitmap.is_used(bitmap_first_frame)
    } {
        debug::write(
            b"ERROR: Bitmap frame is not reserved\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    let bitmap_last_frame =
        bitmap_first_frame + bitmap_pages - 1;

    if !unsafe {
        frame_bitmap.is_used(bitmap_last_frame)
    } {
        debug::write(
            b"ERROR: Bitmap last frame is not reserved\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    for index in 0..memory_map.descriptor_count() {
        let descriptor =
            match unsafe {
                memory_map.descriptor(index)
            } {
                Some(descriptor) => descriptor,

                None => {
                    debug::write(
                        b"ERROR: Failed to read descriptor\r\n"
                    );

                    loop {
                        core::hint::spin_loop();
                    }
                }
            };

        // Descriptor validation/traversal is intentionally
        // preserved even though normal boot logging is disabled.
        let _ = descriptor;
    }

    let mut allocator =
        PhysicalFrameAllocator::new(frame_bitmap);

    if boot_info.userspace_image_size != 0 {
        if allocator
            .reserve_range(
                boot_info.userspace_image_addr,
                boot_info.userspace_image_size,
            )
            .is_err()
        {
            debug::write(
                b"ERROR: Failed to reserve userspace image.\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    debug::write(
        b"Memory initialized.\r\n"
    );

    allocator
}