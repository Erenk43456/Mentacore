#![no_std]
#![no_main]

extern crate alloc;

use alloc::{
    boxed::Box,
    string::String,
    vec::Vec,
};

mod boot_state;
mod cpu;
mod display;
mod interrupts;
mod memory;
mod hardware;

use core::arch::asm;
use core::panic::PanicInfo;

use display::{boot_ui, Framebuffer};
use mentacore_boot_protocol::BootInfo;

use boot_state::BootState;
use memory::memory_map::MemoryMap;
use memory::physical::PhysicalFrameAllocator;

const COM1: u16 = 0x3F8;

const LAPIC_VIRTUAL_BASE: u64 =
    0xFFFF_A000_0000_0000;

unsafe fn serial_write_byte(byte: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") COM1,
            in("al") byte,
            options(nostack, preserves_flags)
        );
    }
}

fn serial_write(message: &[u8]) {
    for &byte in message {
        unsafe {
            serial_write_byte(byte);
        }
    }
}

fn serial_write_hex(value: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    serial_write(b"0x");

    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as usize;

        unsafe {
            serial_write_byte(HEX[digit]);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
    serial_write(b"!!! KERNEL _START REACHED !!!\r\n");
    
    unsafe {
        asm!("cli");
    }

    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"       MENTACORE KERNEL\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"\r\n");

    if boot_info.is_null() {
        serial_write(b"ERROR: BootInfo is NULL\r\n");

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"BootInfo received.\r\n");

    let boot_info = unsafe { &*boot_info };

    serial_write(b"Memory map received.\r\n");

    serial_write(b"  Address: ");
    serial_write_hex(boot_info.memory_map_addr);
    serial_write(b"\r\n");

    serial_write(b"  Size: ");
    serial_write_hex(boot_info.memory_map_size);
    serial_write(b"\r\n");

    serial_write(b"  Descriptor size: ");
    serial_write_hex(boot_info.memory_map_descriptor_size as u64);
    serial_write(b"\r\n");

    serial_write(b"  Descriptor version: ");
    serial_write_hex(boot_info.memory_map_descriptor_version as u64);
    serial_write(b"\r\n");

    serial_write(b"Parsing memory map...\r\n");

    let memory_map = match unsafe {
        MemoryMap::from_boot_info(boot_info)
    } {
        Some(map) => map,

        None => {
            serial_write(b"ERROR: Invalid memory map\r\n");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    let highest_conventional_address =
        match memory_map.highest_conventional_address() {
            Some(address) => address,
            None => {
                serial_write(b"No conventional memory found.\r\n");
                loop {}
            }
        };

    serial_write(b"Highest conventional address: ");
    serial_write_hex(highest_conventional_address);
    serial_write(b"\r\n");

    serial_write(b"Memory map descriptor count: ");
    serial_write_hex(memory_map.descriptor_count() as u64);
    serial_write(b"\r\n");

    let frame_count =
        match memory::physical::frame_count_for_address(
            highest_conventional_address
        ) {
            Some(count) => count,
            None => {
                serial_write(b"Failed to calculate frame count.\r\n");
                loop {}
            }
        };

    let bitmap_size =
        match memory::physical::bitmap_size_bytes(frame_count) {
            Some(size) => size,
            None => {
                serial_write(b"Failed to calculate bitmap size.\r\n");
                loop {}
            }
        };

    let bitmap_pages =
        match memory::physical::bitmap_page_count(frame_count) {
            Some(pages) => pages,
            None => {
                serial_write(b"Failed to calculate bitmap page count.\r\n");
                loop {}
            }
        };

    serial_write(b"Physical frame count: ");
    serial_write_hex(frame_count);
    serial_write(b"\r\n");

    serial_write(b"Bitmap size: ");
    serial_write_hex(bitmap_size);
    serial_write(b" bytes\r\n");

    serial_write(b"Bitmap pages: ");
    serial_write_hex(bitmap_pages);
    serial_write(b"\r\n");

    let bitmap_address =
        match memory_map.find_conventional_region(
            bitmap_pages * memory::paging::PAGE_SIZE,
        ) {
            Some(address) => address,
            None => {
                serial_write(b"Failed to find bitmap region.\r\n");
                loop {}
            }
        };

    serial_write(b"Bitmap physical address: ");
    serial_write_hex(bitmap_address);
    serial_write(b"\r\n");

    let mut frame_bitmap = unsafe {
        match memory::physical::FrameBitmap::new(
            bitmap_address,
            frame_count,
        ) {
            Some(bitmap) => bitmap,
            None => {
                serial_write(b"Failed to create frame bitmap.\r\n");
                loop {}
            }
        }
    };

    unsafe {
        frame_bitmap.clear_all();

        // Start with every physical frame marked as used.
        for frame in 0..frame_count {
            frame_bitmap.set(frame);
        }

        // UEFI Conventional Memory is available.
        for index in 0..memory_map.descriptor_count() {
            let descriptor = match memory_map.descriptor(index) {
                Some(descriptor) => descriptor,
                None => {
                    serial_write(
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
            frame_bitmap.set(bitmap_frame + frame);
        }

        // Physical frame 0 is permanently reserved.
        frame_bitmap.set(0);
    }

    serial_write(b"Frame bitmap initialized.\r\n");

    unsafe {
        if !frame_bitmap.is_used(0) {
            serial_write(b"ERROR: Frame 0 is not reserved\r\n");
            loop {
                core::hint::spin_loop();
            }
        }

        let bitmap_first_frame =
            bitmap_address / memory::paging::PAGE_SIZE;

        if !frame_bitmap.is_used(bitmap_first_frame) {
            serial_write(b"ERROR: Bitmap frame is not reserved\r\n");
            loop {
                core::hint::spin_loop();
            }
        }

        let bitmap_last_frame =
            bitmap_first_frame + bitmap_pages - 1;

        if !frame_bitmap.is_used(bitmap_last_frame) {
            serial_write(b"ERROR: Bitmap last frame is not reserved\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(b"Frame bitmap ownership checks OK.\r\n");

    for index in 0..memory_map.descriptor_count() {
        let descriptor = match unsafe {
            memory_map.descriptor(index)
        } {
            Some(descriptor) => descriptor,

            None => {
                serial_write(b"ERROR: Failed to read descriptor\r\n");

                loop {
                    core::hint::spin_loop();
                }
            }
        };

        serial_write(b"  Descriptor ");
        serial_write_hex(index as u64);

        serial_write(b": type=");
        serial_write_hex(descriptor.ty as u64);

        serial_write(b" physical=");
        serial_write_hex(descriptor.physical_start);

        serial_write(b" pages=");
        serial_write_hex(descriptor.number_of_pages);

        serial_write(b"\r\n");
    }

    serial_write(b"Memory map parsed successfully.\r\n");

    serial_write(b"Initializing physical frame allocator...\r\n");

    let mut allocator =
        PhysicalFrameAllocator::new(frame_bitmap);

    serial_write(b"Physical frame allocator OK.\r\n");

    serial_write(b"Testing physical frame allocation/free...\r\n");

    let test_frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => {
            serial_write(b"ERROR: Failed to allocate test frame\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    serial_write(b"  Allocated test frame: ");
    serial_write_hex(test_frame.start_address);
    serial_write(b"\r\n");

    if allocator
        .is_frame_used(test_frame)
        .unwrap_or(false)
    {
        serial_write(b"  Test frame marked used.\r\n");
    } else {
        serial_write(b"ERROR: Test frame not marked used\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    match allocator.free_frame(test_frame) {
        Ok(()) => {
            serial_write(b"  Test frame freed.\r\n");
        }

        Err(()) => {
            serial_write(b"ERROR: Failed to free test frame\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    }

    if allocator
        .is_frame_used(test_frame)
        .unwrap_or(false)
    {
        serial_write(b"ERROR: Freed frame still marked used\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"PHYSICAL FRAME FREE TEST OK\r\n");

    serial_write(b"Testing physical frame reuse...\r\n");

    let reused_frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => {
            serial_write(b"ERROR: Failed to allocate reused frame\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    if reused_frame.start_address != test_frame.start_address {
        serial_write(b"ERROR: Freed frame was not reused\r\n");
        serial_write(b"Expected: ");
        serial_write_hex(test_frame.start_address);
        serial_write(b"\r\nActual:   ");
        serial_write_hex(reused_frame.start_address);
        serial_write(b"\r\n");

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"  Freed frame was reused: ");
    serial_write_hex(reused_frame.start_address);
    serial_write(b"\r\n");

    serial_write(b"PHYSICAL FRAME REUSE TEST OK\r\n");

    serial_write(b"Testing physical allocator counters...\r\n");

    let live_before = allocator.allocated_count();
    let total_before = allocator.total_allocations();

    let counter_frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => {
            serial_write(b"ERROR: Counter test allocation failed\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    if allocator.allocated_count() != live_before + 1 {
        serial_write(b"ERROR: Live allocation counter did not increment\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    if allocator.total_allocations() != total_before + 1 {
        serial_write(b"ERROR: Total allocation counter did not increment\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    match allocator.free_frame(counter_frame) {
        Ok(()) => {}
        Err(()) => {
            serial_write(b"ERROR: Counter test free failed\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    }

    if allocator.allocated_count() != live_before {
        serial_write(b"ERROR: Live allocation counter did not decrement\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    if allocator.total_allocations() != total_before + 1 {
        serial_write(b"ERROR: Total allocation counter changed after free\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"PHYSICAL FRAME COUNTER TEST OK\r\n");

    serial_write(b"Testing physical frame double-free protection...\r\n");

    let double_free_frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => {
            serial_write(b"ERROR: Double-free test allocation failed\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    };

    match allocator.free_frame(double_free_frame) {
        Ok(()) => {}
        Err(()) => {
            serial_write(b"ERROR: First free failed\r\n");
            loop {
                core::hint::spin_loop();
            }
        }
    }

    // The second free must fail.
    if allocator.free_frame(double_free_frame).is_ok() {
        serial_write(b"ERROR: Double-free was accepted\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"  Double-free correctly rejected.\r\n");
    serial_write(b"PHYSICAL FRAME DOUBLE-FREE TEST OK\r\n");

    serial_write(b"Testing invalid physical frame rejection...\r\n");

    let unaligned_frame =
        memory::physical::Frame {
            start_address: 0x1234,
        };

    if allocator.free_frame(unaligned_frame).is_ok() {
        serial_write(b"ERROR: Unaligned frame was accepted\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"  Unaligned frame correctly rejected.\r\n");

    let out_of_range_address =
        match allocator
            .frame_count()
            .checked_mul(memory::paging::PAGE_SIZE)
        {
            Some(address) => address,
            None => {
                serial_write(b"ERROR: Failed to calculate invalid frame address\r\n");
                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let out_of_range_frame =
        memory::physical::Frame {
            start_address: out_of_range_address,
        };

    if allocator.free_frame(out_of_range_frame).is_ok() {
        serial_write(b"ERROR: Out-of-range frame was accepted\r\n");
        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"  Out-of-range frame correctly rejected.\r\n");
    serial_write(b"PHYSICAL FRAME INVALID TEST OK\r\n");

    serial_write(b"Allocated frames before paging: ");
    serial_write_hex(allocator.allocated_count());
    serial_write(b"\r\n");

    serial_write(b"Initializing paging...\r\n");

    match unsafe {
        memory::paging::init(&mut allocator, boot_info)
    } {
        Ok(()) => {
            serial_write(b"Paging initialized.\r\n");
            
            serial_write(b"Allocated frames after paging: ");
            serial_write_hex(allocator.allocated_count());
            serial_write(b"\r\n");
        }

        Err(()) => {
            serial_write(b"ERROR: Failed to initialize paging\r\n");

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(b"Testing duplicate virtual mapping rejection...\r\n");

    let duplicate_virtual =
        0xFFFF_9000_0000_2000;

    let duplicate_frame_a =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => {
                serial_write(
                    b"ERROR: Failed to allocate duplicate test frame A\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let duplicate_frame_b =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => {
                serial_write(
                    b"ERROR: Failed to allocate duplicate test frame B\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
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
        match mapper.map(
            &mut allocator,
            duplicate_virtual,
            duplicate_frame_a.start_address,
            memory::paging::PageFlags {
                writable: true,
                cache_disable: false,
            },
        ) {
            Ok(()) => {}
            Err(()) => {
                serial_write(
                    b"ERROR: Initial virtual mapping failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    if allocator
        .is_frame_used(duplicate_frame_a)
        .unwrap_or(false)
    {
        serial_write(
            b"  First mapping frame marked used.\r\n"
        );
    } else {
        serial_write(
            b"ERROR: First mapping frame not marked used\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    unsafe {
        if mapper
            .map(
                &mut allocator,
                duplicate_virtual,
                duplicate_frame_b.start_address,
                memory::paging::PageFlags {
                    writable: true,
                    cache_disable: false,
                },
            )
            .is_ok()
        {
            serial_write(
                b"ERROR: Duplicate virtual mapping was accepted\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Duplicate virtual mapping correctly rejected.\r\n"
    );

    if allocator
        .is_frame_used(duplicate_frame_b)
        .unwrap_or(false)
    {
        serial_write(
            b"  Replacement frame remains allocated.\r\n"
        );
    } else {
        serial_write(
            b"ERROR: Replacement frame became unexpectedly free\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(
        b"PAGING DUPLICATE MAP TEST OK\r\n"
    );

    serial_write(
        b"Testing virtual-to-physical mapping...\r\n"
    );

    let mapping_virtual =
        0xFFFF_9000_0000_3000;

    let mapping_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => {
                serial_write(
                    b"ERROR: Failed to allocate mapping test frame\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
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
        match mapper.map(
            &mut allocator,
            mapping_virtual,
            mapping_physical,
            memory::paging::PageFlags {
                writable: true,
                cache_disable: false,
            },
        ) {
            Ok(()) => {}

            Err(()) => {
                serial_write(
                    b"ERROR: Mapping test page failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    let virtual_ptr =
        mapping_virtual as *mut u64;

    unsafe {
        virtual_ptr.write(
            0xAABB_CCDD_1122_3344
        );

        if virtual_ptr.read()
            != 0xAABB_CCDD_1122_3344
        {
            serial_write(
                b"ERROR: Virtual mapping read/write failed\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Virtual address read/write OK.\r\n"
    );

    let physical_ptr =
        mapping_physical as *const u64;

    unsafe {
        if physical_ptr.read()
            != 0xAABB_CCDD_1122_3344
        {
            serial_write(
                b"ERROR: Physical frame contents mismatch\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Physical frame contents match.\r\n"
    );

    serial_write(
        b"PAGING MAPPING TEST OK\r\n"
    );

    serial_write(
        b"Testing virtual page unmapping...\r\n"
    );

    let unmap_virtual =
        0xFFFF_9000_0000_4000;

    let unmap_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => {
                serial_write(
                    b"ERROR: Failed to allocate unmap test frame\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
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
        match mapper.map(
            &mut allocator,
            unmap_virtual,
            unmap_physical,
            memory::paging::PageFlags {
                writable: true,
                cache_disable: false,
            },
        ) {
            Ok(()) => {}

            Err(()) => {
                serial_write(
                    b"ERROR: Unmap test initial mapping failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    let unmap_ptr =
        unmap_virtual as *mut u64;

    unsafe {
        unmap_ptr.write(
            0x5566_7788_AABB_CCDD
        );

        if unmap_ptr.read()
            != 0x5566_7788_AABB_CCDD
        {
            serial_write(
                b"ERROR: Unmap test initial access failed\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Initial mapping access OK.\r\n"
    );

    let unmapped_physical =
        unsafe {
            match mapper.unmap(
                unmap_virtual,
            ) {
                Ok(address) => address,

                Err(()) => {
                    serial_write(
                        b"ERROR: Mapper::unmap failed\r\n"
                    );

                    loop {
                        core::hint::spin_loop();
                    }
                }
            }
        };

    if unmapped_physical != unmap_physical {
        serial_write(
            b"ERROR: Unmapped physical address mismatch\r\n"
        );

        serial_write(b"Expected: ");
        serial_write_hex(unmap_physical);
        serial_write(b"\r\n");

        serial_write(b"Actual:   ");
        serial_write_hex(unmapped_physical);
        serial_write(b"\r\n");

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(
        b"  Correct physical frame returned.\r\n"
    );

    if !allocator
        .is_frame_used(unmap_frame)
        .unwrap_or(false)
    {
        serial_write(
            b"ERROR: Unmapped frame became free unexpectedly\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(
        b"  Unmapped frame remains allocated.\r\n"
    );

    let returned_frame =
        match memory::physical::Frame::new(
            unmapped_physical
        ) {
            Some(frame) => frame,

            None => {
                serial_write(
                    b"ERROR: Returned physical address is not a valid frame\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    match allocator.free_frame(returned_frame) {
        Ok(()) => {}

        Err(()) => {
            serial_write(
                b"ERROR: Failed to free unmapped frame\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    if allocator
        .is_frame_used(returned_frame)
        .unwrap_or(true)
    {
        serial_write(
            b"ERROR: Unmapped frame still marked used after free\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(
        b"  Unmapped frame freed successfully.\r\n"
    );

    let reused_frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,

            None => {
                serial_write(
                    b"ERROR: Failed to reallocate freed frame\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    if reused_frame.start_address != unmap_physical {
        serial_write(
            b"ERROR: Freed unmapped frame was not reused\r\n"
        );

        serial_write(b"Expected: ");
        serial_write_hex(unmap_physical);
        serial_write(b"\r\n");

        serial_write(b"Actual:   ");
        serial_write_hex(reused_frame.start_address);
        serial_write(b"\r\n");

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(
        b"  Freed frame reused successfully.\r\n"
    );

    let remap_physical =
        reused_frame.start_address;

    unsafe {
        let mut mapper =
            memory::paging::Mapper::new(pml4);

        match mapper.map(
            &mut allocator,
            unmap_virtual,
            remap_physical,
            memory::paging::PageFlags {
                writable: true,
                cache_disable: false,
            },
        ) {
            Ok(()) => {}

            Err(()) => {
                serial_write(
                    b"ERROR: Remapping unmapped virtual page failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    unsafe {
        unmap_ptr.write(
            0x1122_3344_5566_7788
        );

        if unmap_ptr.read()
            != 0x1122_3344_5566_7788
        {
            serial_write(
                b"ERROR: Remapped page access failed\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Remapped page access OK.\r\n"
    );

    serial_write(
        b"PAGING UNMAP TEST OK\r\n"
    );

    serial_write(
        b"Testing virtual page unmapping rejection paths...\r\n"
    );

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    // ---------------------------------------------------------
    // 1. Non-canonical virtual address
    // ---------------------------------------------------------

    let noncanonical_virtual =
        0x0000_8000_0000_0000;

    unsafe {
        if mapper
            .unmap(
                noncanonical_virtual,
            )
            .is_ok()
        {
            serial_write(
                b"ERROR: Non-canonical unmap was accepted\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Non-canonical address correctly rejected.\r\n"
    );

    // ---------------------------------------------------------
    // 2. Unaligned virtual address
    // ---------------------------------------------------------

    let unaligned_virtual =
        0xFFFF_9000_0000_5001;

    unsafe {
        if mapper
            .unmap(
                unaligned_virtual,
            )
            .is_ok()
        {
            serial_write(
                b"ERROR: Unaligned unmap was accepted\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Unaligned address correctly rejected.\r\n"
    );

    // ---------------------------------------------------------
    // 3. Valid but never-mapped virtual address
    // ---------------------------------------------------------

    let unmapped_virtual =
        0xFFFF_9000_0000_6000;

    unsafe {
        if mapper
            .unmap(
                unmapped_virtual,
            )
            .is_ok()
        {
            serial_write(
                b"ERROR: Unmapped page was accepted\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Unmapped page correctly rejected.\r\n"
    );

    // ---------------------------------------------------------
    // 4. Huge-page mapping
    // ---------------------------------------------------------
    //
    // The identity map uses 2 MiB huge pages.
    // Attempting to unmap one through the 4 KiB unmap API
    // must be rejected.
    //

    let huge_page_virtual =
        0x0000_0020_0000;

    unsafe {
        if mapper
            .unmap(
                huge_page_virtual,
            )
            .is_ok()
        {
            serial_write(
                b"ERROR: Huge-page unmap was accepted\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }

    serial_write(
        b"  Huge-page mapping correctly rejected.\r\n"
    );

    serial_write(
        b"PAGING UNMAP REJECTION TEST OK\r\n"
    );

    serial_write(
        b"Initializing kernel heap...\r\n"
    );

    unsafe {
        memory::heap::init();
    }

    serial_write(
        b"Initializing CPU state...\r\n"
    );

    cpu::init();

    serial_write(
        b"CPU state initialized.\r\n"
    );

    serial_write(b"Initializing LAPIC...\r\n");

    let mut lapic =
        match hardware::lapic::Lapic::discover() {
            Some(lapic) => lapic,

            None => {
                serial_write(
                    b"LAPIC DISCOVERY FAILED\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    serial_write(b"LAPIC MSR: ");
    serial_write_hex(lapic.msr_value());
    serial_write(b"\r\n");

    serial_write(b"LAPIC physical base: ");
    serial_write_hex(lapic.physical_base());
    serial_write(b"\r\n");

    serial_write(b"LAPIC enabled: YES\r\n");

    serial_write(b"Mapping LAPIC MMIO...\r\n");

    let pml4 =
        unsafe {
            memory::paging::current_pml4()
        };

    let mut lapic_mapper =
        unsafe {
            memory::paging::Mapper::new(pml4)
        };

    unsafe {
        match lapic_mapper.map(
            &mut allocator,
            LAPIC_VIRTUAL_BASE,
            lapic.physical_base(),
            memory::paging::PageFlags {
                writable: true,
                cache_disable: true,
            },
        ) {
            Ok(()) => {}

            Err(()) => {
                serial_write(
                    b"ERROR: LAPIC MMIO mapping failed\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    lapic.set_virtual_base(LAPIC_VIRTUAL_BASE);

    serial_write(b"LAPIC virtual base: ");
    serial_write_hex(lapic.virtual_base());
    serial_write(b"\r\n");

    serial_write(b"LAPIC MMIO mapping OK\r\n");

    let lapic_id =
        unsafe {
            lapic.id()
        };

    serial_write(b"LAPIC ID register: ");
    serial_write_hex(lapic_id as u64);
    serial_write(b"\r\n");

    let lapic_version =
        unsafe {
            lapic.version()
        };

    serial_write(b"LAPIC VERSION register: ");
    serial_write_hex(lapic_version as u64);
    serial_write(b"\r\n");

    let mut lapic_svr =
        unsafe {
            lapic.svr()
        };

    serial_write(b"LAPIC SVR register: ");
    serial_write_hex(lapic_svr as u64);
    serial_write(b"\r\n");

    if (lapic_svr & hardware::lapic::LAPIC_SVR_ENABLE) == 0 {
        serial_write(b"Enabling LAPIC software...\r\n");

        unsafe {
            lapic.set_svr(lapic_svr | hardware::lapic::LAPIC_SVR_ENABLE);
        }

        lapic_svr =
            unsafe {
                lapic.svr()
            };

        serial_write(b"LAPIC SVR after enable: ");
        serial_write_hex(lapic_svr as u64);
        serial_write(b"\r\n");
    }

    let lapic_lvt_timer =
        unsafe {
            lapic.lvt_timer()
        };

    serial_write(b"LAPIC LVT TIMER register: ");
    serial_write_hex(lapic_lvt_timer as u64);
    serial_write(b"\r\n");

    let lapic_lvt_error =
        unsafe {
            lapic.lvt_error()
        };

    serial_write(b"LAPIC LVT ERROR register: ");
    serial_write_hex(lapic_lvt_error as u64);
    serial_write(b"\r\n");

    serial_write(b"LAPIC SOFTWARE ENABLED\r\n");
    serial_write(b"LAPIC REGISTER ACCESS OK\r\n");

    serial_write(
        b"Initializing interrupt system...\r\n"
    );

    unsafe {
        interrupts::init(&mut allocator);
    }

    serial_write(b"Interrupt system initialized.\r\n");

    serial_write(b"Initializing PIC...\r\n");

    unsafe {
        hardware::pic::remap();
        hardware::pic::enable_irq(0);
    }

    serial_write(b"PIC initialized.\r\n");

    serial_write(b"Initializing PIT...\r\n");

    unsafe {
        hardware::pit::set_frequency(100);
    }

    serial_write(b"PIT initialized.\r\n");

    serial_write(b"Kernel heap initialized.\r\n");

    serial_write(b"Testing kernel heap...\r\n");

    let mut values = Vec::new();
    values.push(10u64);
    values.push(20u64);
    values.push(30u64);

    let boxed = Box::new(1234u64);

    let text = String::from("Mentacore heap");

    serial_write(b"  String: ");
    serial_write(text.as_bytes());
    serial_write(b"\r\n");

    serial_write(b"  Vec[0]: ");
    serial_write_hex(values[0]);
    serial_write(b"\r\n");

    serial_write(b"  Vec[1]: ");
    serial_write_hex(values[1]);
    serial_write(b"\r\n");

    serial_write(b"  Vec[2]: ");
    serial_write_hex(values[2]);
    serial_write(b"\r\n");

    serial_write(b"  Box: ");
    serial_write_hex(*boxed);
    serial_write(b"\r\n");

    serial_write(b"  String allocated.\r\n");

    serial_write(b"HEAP TEST OK\r\n");

    serial_write(b"Testing multi-page heap...\r\n");

    let mut multi_page = Vec::with_capacity(2048);

    for i in 0..2048u64 {
        multi_page.push(i);
    }

    serial_write(b"  Multi-page Vec allocated.\r\n");

    serial_write(b"  Multi-page Vec[0]: ");
    serial_write_hex(multi_page[0]);
    serial_write(b"\r\n");

    serial_write(b"  Multi-page Vec[1024]: ");
    serial_write_hex(multi_page[1024]);
    serial_write(b"\r\n");

    serial_write(b"  Multi-page Vec[2047]: ");
    serial_write_hex(multi_page[2047]);
    serial_write(b"\r\n");

    serial_write(b"HEAP MULTI-PAGE TEST OK\r\n");

    serial_write(b"Initializing display renderer...\r\n");

    let mut framebuffer = unsafe {
        Framebuffer::from_boot_info(boot_info)
    };

    serial_write(b"Display renderer initialized.\r\n");

    let state = BootState::from_boot_info(boot_info);

    serial_write(b"Rendering Mentacore boot UI...\r\n");

    boot_ui::render(
        &mut framebuffer,
        state,
    );

    serial_write(b"DISPLAY OK\r\n");

    serial_write(b"Enabling hardware interrupts...\r\n");
    unsafe {
        asm!("sti");
    }
    serial_write(b"Hardware interrupts enabled.\r\n");

    serial_write(b"Calibrating TSC...\r\n");

    let tsc_frequency = cpu::calibrate_tsc();

    serial_write(b"TSC frequency: ");
    serial_write_hex(tsc_frequency);
    serial_write(b" Hz\r\n");

    if tsc_frequency != 0 {
        serial_write(b"TSC CALIBRATION OK\r\n");
    } else {
        serial_write(b"TSC CALIBRATION FAILED\r\n");
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_write(b"MENTACORE KERNEL PANIC\r\n");

    loop {
        core::hint::spin_loop();
    }
}