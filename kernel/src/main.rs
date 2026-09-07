#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

use mentacore_boot_protocol::BootInfo;

const COM1: u16 = 0x3F8;

const FONT_WIDTH: usize = 8;
const FONT_HEIGHT: usize = 8;

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

struct Framebuffer {
    address: *mut u8,
    size: usize,
    width: usize,
    height: usize,
    stride: usize,
    format: u32,
}

impl Framebuffer {
    unsafe fn from_boot_info(boot_info: &BootInfo) -> Self {
        Self {
            address: boot_info.framebuffer_addr as *mut u8,
            size: boot_info.framebuffer_size as usize,
            width: boot_info.framebuffer_width as usize,
            height: boot_info.framebuffer_height as usize,
            stride: boot_info.framebuffer_stride as usize,
            format: boot_info.framebuffer_format,
        }
    }

    unsafe fn write_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = (y * self.stride + x) * 4;

        if offset + 4 > self.size {
            return;
        }

        let pixel = match self.format {
            // RGB
            0 => [color[0], color[1], color[2], 0],

            // BGR
            1 => [color[2], color[1], color[0], 0],

            // Bitmask
            2 => [color[0], color[1], color[2], 0],

            // BltOnly cannot be written directly.
            3 => return,

            _ => return,
        };

        unsafe {
            self.address.add(offset).write_volatile(pixel[0]);
            self.address.add(offset + 1).write_volatile(pixel[1]);
            self.address.add(offset + 2).write_volatile(pixel[2]);
            self.address.add(offset + 3).write_volatile(pixel[3]);
        }
    }

    unsafe fn clear(&mut self, color: [u8; 3]) {
        for y in 0..self.height {
            for x in 0..self.width {
                unsafe {
                    self.write_pixel(x, y, color);
                }
            }
        }
    }

    unsafe fn draw_char(
        &mut self,
        x: usize,
        y: usize,
        character: u8,
        color: [u8; 3],
    ) {
        let glyph = glyph(character);

        for row in 0..FONT_HEIGHT {
            let bits = glyph[row];

            for col in 0..FONT_WIDTH {
                if bits & (1 << (7 - col)) != 0 {
                    unsafe {
                        self.write_pixel(x + col, y + row, color);
                    }
                }
            }
        }
    }

    unsafe fn draw_string(
        &mut self,
        x: usize,
        y: usize,
        text: &[u8],
        color: [u8; 3],
    ) {
        let mut cursor_x = x;

        for &character in text {
            unsafe {
                self.draw_char(cursor_x, y, character, color);
            }

            cursor_x += FONT_WIDTH;
        }
    }
}

fn glyph(character: u8) -> [u8; 8] {
    match character {
        b' ' => [
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
            0b00000000,
        ],

        b'A' => [
            0b00011000,
            0b00111100,
            0b01100110,
            0b01100110,
            0b01111110,
            0b01100110,
            0b01100110,
            0b00000000,
        ],

        b'B' => [
            0b01111100,
            0b01100110,
            0b01100110,
            0b01111100,
            0b01100110,
            0b01100110,
            0b01111100,
            0b00000000,
        ],

        b'C' => [
            0b00111110,
            0b01100000,
            0b11000000,
            0b11000000,
            0b11000000,
            0b01100000,
            0b00111110,
            0b00000000,
        ],

        b'E' => [
            0b01111110,
            0b01100000,
            0b01100000,
            0b01111100,
            0b01100000,
            0b01100000,
            0b01111110,
            0b00000000,
        ],

        b'G' => [
            0b00111110,
            0b01100000,
            0b11000000,
            0b11001110,
            0b11000110,
            0b01100110,
            0b00111110,
            0b00000000,
        ],

        b'K' => [
            0b11000110,
            0b11001100,
            0b11011000,
            0b11110000,
            0b11110000,
            0b11011000,
            0b11001100,
            0b00000000,
        ],

        b'L' => [
            0b01100000,
            0b01100000,
            0b01100000,
            0b01100000,
            0b01100000,
            0b01100000,
            0b01111110,
            0b00000000,
        ],

        b'M' => [
            0b11000011,
            0b11100111,
            0b11111111,
            0b11011011,
            0b11000011,
            0b11000011,
            0b11000011,
            0b00000000,
        ],

        b'N' => [
            0b11000011,
            0b11100011,
            0b11110011,
            0b11011011,
            0b11001111,
            0b11000111,
            0b11000011,
            0b00000000,
        ],

        b'O' => [
            0b00111100,
            0b01100110,
            0b11000011,
            0b11000011,
            0b11000011,
            0b01100110,
            0b00111100,
            0b00000000,
        ],

        b'R' => [
            0b01111100,
            0b01100110,
            0b01100110,
            0b01111100,
            0b01101100,
            0b01100110,
            0b01100110,
            0b00000000,
        ],

        b'T' => [
            0b01111110,
            0b00011000,
            0b00011000,
            0b00011000,
            0b00011000,
            0b00011000,
            0b00011000,
            0b00000000,
        ],

        b'X' => [
            0b11000011,
            0b01100110,
            0b00111100,
            0b00011000,
            0b00111100,
            0b01100110,
            0b11000011,
            0b00000000,
        ],

        b'0' => [
            0b00111100,
            0b01100110,
            0b01101110,
            0b01110110,
            0b01100110,
            0b01100110,
            0b00111100,
            0b00000000,
        ],

        b'1' => [
            0b00011000,
            0b00111000,
            0b00011000,
            0b00011000,
            0b00011000,
            0b00011000,
            0b01111110,
            0b00000000,
        ],

        b'2' => [
            0b00111100,
            0b01100110,
            0b00000110,
            0b00001100,
            0b00011000,
            0b00110000,
            0b01111110,
            0b00000000,
        ],

        b'8' => [
            0b00111100,
            0b01100110,
            0b01100110,
            0b00111100,
            0b01100110,
            0b01100110,
            0b00111100,
            0b00000000,
        ],

        _ => [
            0b11111111,
            0b10000001,
            0b10100101,
            0b10000001,
            0b10100101,
            0b10011001,
            0b10000001,
            0b11111111,
        ],
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

    let mut framebuffer = unsafe { Framebuffer::from_boot_info(boot_info) };

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