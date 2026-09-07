use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    text::Text,
};

use super::style::TextStyle;
use super::super::Framebuffer;

pub struct TextLabel {
    position: Point,
    text: &'static str,
    style: TextStyle,
}

impl TextLabel {
    pub fn new(
        position: Point,
        text: &'static str,
        style: TextStyle,
    ) -> Self {
        Self {
            position,
            text,
            style,
        }
    }

    pub fn draw(
        &self,
        framebuffer: &mut Framebuffer,
    ) {
        let text_style = MonoTextStyle::new(
            self.style.font,
            self.style.color,
        );

        Text::new(
            self.text,
            self.position,
            text_style,
        )
        .draw(framebuffer)
        .unwrap();
    }
}