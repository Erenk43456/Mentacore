#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

use mentacore_boot_protocol::BootInfo;

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

unsafe fn framebuffer_fill(boot_info: &BootInfo) {
    let width = boot_info.framebuffer_width as usize;
    let height = boot_info.framebuffer_height as usize;
    let stride = boot_info.framebuffer_stride as usize;

    let framebuffer = boot_info.framebuffer_addr as *mut u8;
    let framebuffer_size = boot_info.framebuffer_size as usize;

    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x) * 4;

            if offset + 4 > framebuffer_size {
                return;
            }

            let pixel = match boot_info.framebuffer_format {
                // RGB
                0 => [0x20, 0x40, 0x80, 0x00],

                // BGR
                1 => [0x80, 0x40, 0x20, 0x00],

                // Bitmask / BltOnly
                _ => [0x20, 0x40, 0x80, 0x00],
            };

            unsafe {
                framebuffer
                    .add(offset)
                    .write_volatile(pixel[0]);

                framebuffer
                    .add(offset + 1)
                    .write_volatile(pixel[1]);

                framebuffer
                    .add(offset + 2)
                    .write_volatile(pixel[2]);

                framebuffer
                    .add(offset + 3)
                    .write_volatile(pixel[3]);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start(boot_info: *const BootInfo) -> ! {
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

    serial_write(b"Drawing framebuffer...\r\n");

    unsafe {
        framebuffer_fill(boot_info);
    }

    serial_write(b"FRAMEBUFFER OK\r\n");

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