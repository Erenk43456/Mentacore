#![no_std]
#![no_main]

mod boot_state;
mod display;
mod memory_map;
mod physical_memory;

use core::arch::asm;
use core::panic::PanicInfo;

use display::{boot_ui, Framebuffer};
use mentacore_boot_protocol::BootInfo;

use boot_state::BootState;
use memory_map::MemoryMap;
use physical_memory::PhysicalFrameAllocator;

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

    serial_write(b"Memory map descriptor count: ");
    serial_write_hex(memory_map.descriptor_count() as u64);
    serial_write(b"\r\n");

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

    let mut allocator = PhysicalFrameAllocator::new(memory_map);

    for index in 0..10 {
        match allocator.allocate_frame() {
            Some(frame) => {
                serial_write(b"  Frame ");
                serial_write_hex(index);
                serial_write(b": ");
                serial_write_hex(frame.start_address);
                serial_write(b"\r\n");
            }

            None => {
                serial_write(b"ERROR: No physical frames available\r\n");

                loop {
                    core::hint::spin_loop();
                }
            }
        }
    }

    serial_write(b"Physical frame allocator OK.\r\n");

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