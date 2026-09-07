use embedded_graphics::{
    geometry::Size,
    prelude::*,
};

use super::Framebuffer;

pub struct ScreenGeometry {
    size: Size,
}

impl ScreenGeometry {
    pub fn new(framebuffer: &Framebuffer) -> Self {
        Self {
            size: framebuffer.size(),
        }
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn center_x(&self) -> i32 {
        (self.width() / 2) as i32
    }

    pub fn center_y(&self) -> i32 {
        (self.height() / 2) as i32
    }

    pub fn centered_x(&self, width: u32) -> i32 {
        self.center_x() - (width / 2) as i32
    }
}

pub struct BootLayout {
    screen: ScreenGeometry,
    top: i32,
}

impl BootLayout {
    pub fn new(framebuffer: &Framebuffer) -> Self {
        let screen = ScreenGeometry::new(framebuffer);

        let content_height = 360;
        let top = screen.center_y() - content_height / 2;

        Self {
            screen,
            top,
        }
    }

    pub fn screen(&self) -> &ScreenGeometry {
        &self.screen
    }

    pub fn header_y(&self) -> i32 {
        self.top
    }

    pub fn subtitle_y(&self) -> i32 {
        self.top + 32
    }

    pub fn header_separator_y(&self) -> i32 {
        self.top + 70
    }

    pub fn status_y(&self) -> i32 {
        self.top + 110
    }

    pub fn status_text_y(&self) -> i32 {
        self.top + 120
    }

    pub fn info_y(&self, index: i32) -> i32 {
        self.top + 180 + index * 32
    }

    pub fn footer_separator_y(&self) -> i32 {
        self.top + 320
    }

    pub fn footer_y(&self) -> i32 {
        self.top + 360
    }
}