#![no_std]
#![no_main]

extern crate alloc;

use alloc::{
    boxed::Box,
    string::String,
    vec::Vec,
};

mod boot_state;
mod display;
mod interrupts;
mod memory;

use core::arch::asm;
use core::panic::PanicInfo;

use display::{boot_ui, Framebuffer};
use mentacore_boot_protocol::BootInfo;

use boot_state::BootState;
use memory::memory_map::MemoryMap;
use memory::physical::PhysicalFrameAllocator;

const COM1: u16 = 0x3F8;

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

    serial_write(b"Initializing kernel heap...\r\n");

    unsafe {
        memory::heap::init();
    }

    serial_write(b"Initializing interrupt system...\r\n");

    unsafe {
        interrupts::init(&mut allocator);
    }

    serial_write(b"Interrupt system initialized.\r\n");

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