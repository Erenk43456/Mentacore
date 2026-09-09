use mentacore_boot_protocol::BootInfo;

use crate::debug;
use crate::memory;
use crate::memory::memory_map::MemoryMap;
use crate::memory::physical::PhysicalFrameAllocator;

pub unsafe fn initialize(
    boot_info: &BootInfo,
) -> PhysicalFrameAllocator {
    debug::write(b"Memory map received.\r\n");

    debug::write(b"  Address: ");
    debug::write_hex(boot_info.memory_map_addr);
    debug::write(b"\r\n");

    debug::write(b"  Size: ");
    debug::write_hex(boot_info.memory_map_size);
    debug::write(b"\r\n");

    debug::write(b"  Descriptor size: ");
    debug::write_hex(
        boot_info.memory_map_descriptor_size as u64,
    );
    debug::write(b"\r\n");

    debug::write(b"  Descriptor version: ");
    debug::write_hex(
        boot_info.memory_map_descriptor_version as u64,
    );
    debug::write(b"\r\n");

    debug::write(b"Parsing memory map...\r\n");

    let memory_map =
        match MemoryMap::from_boot_info(boot_info) {
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
                    b"No conventional memory found.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    debug::write(
        b"Highest conventional address: "
    );
    debug::write_hex(
        highest_conventional_address
    );
    debug::write(b"\r\n");

    debug::write(
        b"Memory map descriptor count: "
    );
    debug::write_hex(
        memory_map.descriptor_count() as u64
    );
    debug::write(b"\r\n");

    let frame_count =
        match memory::physical::frame_count_for_address(
            highest_conventional_address,
        ) {
            Some(count) => count,

            None => {
                debug::write(
                    b"Failed to calculate frame count.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let bitmap_size =
        match memory::physical::bitmap_size_bytes(
            frame_count,
        ) {
            Some(size) => size,

            None => {
                debug::write(
                    b"Failed to calculate bitmap size.\r\n"
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
                    b"Failed to calculate bitmap page count.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    debug::write(b"Physical frame count: ");
    debug::write_hex(frame_count);
    debug::write(b"\r\n");

    debug::write(b"Bitmap size: ");
    debug::write_hex(bitmap_size);
    debug::write(b" bytes\r\n");

    debug::write(b"Bitmap pages: ");
    debug::write_hex(bitmap_pages);
    debug::write(b"\r\n");

    let bitmap_address =
        match memory_map.find_conventional_region(
            bitmap_pages * memory::paging::PAGE_SIZE,
        ) {
            Some(address) => address,

            None => {
                debug::write(
                    b"Failed to find bitmap region.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    debug::write(
        b"Bitmap physical address: "
    );
    debug::write_hex(bitmap_address);
    debug::write(b"\r\n");

    let mut frame_bitmap = match
        memory::physical::FrameBitmap::new(
            bitmap_address,
            frame_count,
        ) {
        Some(bitmap) => bitmap,

        None => {
            debug::write(
                b"Failed to create frame bitmap.\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    };

    frame_bitmap.clear_all();

    // Start with every physical frame marked as used.
    for frame in 0..frame_count {
        frame_bitmap.set(frame);
    }

    // UEFI Conventional Memory is available.
    for index in 0..memory_map.descriptor_count() {
        let descriptor =
            match memory_map.descriptor(index) {
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
            frame_bitmap.mark_free_range(
                descriptor.physical_start,
                descriptor.number_of_pages,
            );
        }
    }

    // The bitmap's own physical pages must remain reserved.
    let bitmap_frame =
        bitmap_address / memory::paging::PAGE_SIZE;

    for frame in 0..bitmap_pages {
        frame_bitmap.set(
            bitmap_frame + frame
        );
    }

    // Physical frame 0 is permanently reserved.
    frame_bitmap.set(0);

    debug::write(
        b"Frame bitmap initialized.\r\n"
    );

    if !frame_bitmap.is_used(0) {
        debug::write(
            b"ERROR: Frame 0 is not reserved\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    let bitmap_first_frame =
        bitmap_address / memory::paging::PAGE_SIZE;

    if !frame_bitmap.is_used(bitmap_first_frame) {
        debug::write(
            b"ERROR: Bitmap frame is not reserved\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    let bitmap_last_frame =
        bitmap_first_frame + bitmap_pages - 1;

    if !frame_bitmap.is_used(bitmap_last_frame) {
        debug::write(
            b"ERROR: Bitmap last frame is not reserved\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    debug::write(
        b"Frame bitmap ownership checks OK.\r\n"
    );

    for index in 0..memory_map.descriptor_count() {
        let descriptor =
            match memory_map.descriptor(index) {
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

        debug::write(b"  Descriptor ");
        debug::write_hex(index as u64);

        debug::write(b": type=");
        debug::write_hex(descriptor.ty as u64);

        debug::write(b" physical=");
        debug::write_hex(descriptor.physical_start);

        debug::write(b" pages=");
        debug::write_hex(descriptor.number_of_pages);

        debug::write(b"\r\n");
    }

    debug::write(
        b"Memory map parsed successfully.\r\n"
    );

    debug::write(
        b"Initializing physical frame allocator...\r\n"
    );

    let allocator =
        PhysicalFrameAllocator::new(frame_bitmap);

    debug::write(
        b"Physical frame allocator OK.\r\n"
    );

    allocator
}