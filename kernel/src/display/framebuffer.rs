use mentacore_boot_protocol::BootInfo;

use super::font::{glyph, FONT_HEIGHT, FONT_WIDTH};

pub struct Framebuffer {
    address: *mut u8,
    size: usize,
    width: usize,
    height: usize,
    stride: usize,
    format: u32,
}

impl Framebuffer {
    pub unsafe fn from_boot_info(boot_info: &BootInfo) -> Self {
        Self {
            address: boot_info.framebuffer_addr as *mut u8,
            size: boot_info.framebuffer_size as usize,
            width: boot_info.framebuffer_width as usize,
            height: boot_info.framebuffer_height as usize,
            stride: boot_info.framebuffer_stride as usize,
            format: boot_info.framebuffer_format,
        }
    }

    pub unsafe fn clear(&mut self, color: [u8; 3]) {
        for y in 0..self.height {
            for x in 0..self.width {
                unsafe {
                    self.write_pixel(x, y, color);
                }
            }
        }
    }

    pub unsafe fn draw_char(
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

    pub unsafe fn draw_string(
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

    unsafe fn write_pixel(
        &mut self,
        x: usize,
        y: usize,
        color: [u8; 3],
    ) {
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

            // BltOnly
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
}