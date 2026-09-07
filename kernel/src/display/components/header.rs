use embedded_graphics::{
    prelude::*,
};

use super::{
    super::Framebuffer,
    TextLabel,
    TextStyle,
};

pub struct Header {
    position: Point,
    title: &'static str,
}

impl Header {
    pub fn new(
        position: Point,
        title: &'static str,
    ) -> Self {
        Self {
            position,
            title,
        }
    }

    pub fn draw(
        &self,
        framebuffer: &mut Framebuffer,
    ) {
        let title = TextLabel::new(
            self.position,
            self.title,
            TextStyle::TITLE,
        );

        title.draw(framebuffer);
    }
}