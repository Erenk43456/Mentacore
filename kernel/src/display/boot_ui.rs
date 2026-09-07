use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
};

use super::Framebuffer;

pub fn render(framebuffer: &mut Framebuffer) {
    framebuffer
        .clear(Rgb888::new(0xFF, 0x00, 0x00))
        .unwrap();

    Rectangle::new(
        Point::new(100, 100),
        Size::new(500, 300),
    )
    .into_styled(
        PrimitiveStyle::with_fill(
            Rgb888::new(0x00, 0xFF, 0x00),
        ),
    )
    .draw(framebuffer)
    .unwrap();
}