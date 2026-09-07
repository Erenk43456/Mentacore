#![no_std]
#![no_main]

mod display;

use core::arch::asm;
use core::panic::PanicInfo;

use display::Framebuffer;
use mentacore_boot_protocol::BootInfo;

const COM1: u16 = 0x3F8;

const BACKGROUND: [u8; 3] = [0x20, 0x40, 0x80];
const FOREGROUND: [u8; 3] = [0xFF, 0xFF, 0xFF];

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

    serial_write(b"Initializing display renderer...\r\n");

    let mut framebuffer = unsafe {
        Framebuffer::from_boot_info(boot_info)
    };

    serial_write(b"Clearing framebuffer...\r\n");

    unsafe {
        framebuffer.clear(BACKGROUND);
    }

    serial_write(b"Drawing kernel text...\r\n");

    unsafe {
        framebuffer.draw_string(
            480,
            300,
            b"MENTACORE KERNEL OK",
            FOREGROUND,
        );

        framebuffer.draw_string(
            500,
            320,
            b"1280X800",
            FOREGROUND,
        );

        framebuffer.draw_string(
            500,
            340,
            b"BGR",
            FOREGROUND,
        );
    }

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