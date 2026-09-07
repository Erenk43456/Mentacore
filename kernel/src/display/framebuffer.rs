use core::convert::Infallible;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::Rgb888,
    prelude::*,
    Pixel,
};

use mentacore_boot_protocol::BootInfo;

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

    unsafe fn write_pixel(
        &mut self,
        x: usize,
        y: usize,
        color: Rgb888,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = (y * self.stride + x) * 4;

        if offset + 4 > self.size {
            return;
        }

        let red = color.r();
        let green = color.g();
        let blue = color.b();

        let pixel = match self.format {
            // RGB
            0 => [red, green, blue, 0],

            // BGR
            1 => [blue, green, red, 0],

            // Bitmask
            2 => [red, green, blue, 0],

            // BltOnly
            3 => return,

            _ => return,
        };

        unsafe {
            self.address
                .add(offset)
                .write_volatile(pixel[0]);

            self.address
                .add(offset + 1)
                .write_volatile(pixel[1]);

            self.address
                .add(offset + 2)
                .write_volatile(pixel[2]);

            self.address
                .add(offset + 3)
                .write_volatile(pixel[3]);
        }
    }
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(
            self.width as u32,
            self.height as u32,
        )
    }
}

impl DrawTarget for Framebuffer {
    type Color = Rgb888;
    type Error = Infallible;

    fn draw_iter<I>(
        &mut self,
        pixels: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x < 0 || point.y < 0 {
                continue;
            }

            unsafe {
                self.write_pixel(
                    point.x as usize,
                    point.y as usize,
                    color,
                );
            }
        }

        Ok(())
    }
}