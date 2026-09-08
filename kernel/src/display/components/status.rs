use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        Circle,
        PrimitiveStyle,
    },
};

use super::super::{
    color,
    Framebuffer,
};

#[derive(Clone, Copy)]
pub enum Status {
    Initializing,
    Ready,
    Warning,
    Error,
}

impl Status {
    pub fn color(self) -> Rgb888 {
        match self {
            Self::Initializing => color::ACCENT,
            Self::Ready => color::SUCCESS,
            Self::Warning => color::WARNING,
            Self::Error => color::ERROR,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Self::Initializing => "INITIALIZING",
            Self::Ready => "SYSTEM READY",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
        }
    }

    pub fn text_style(self) -> super::TextStyle {
        match self {
            Self::Initializing => super::TextStyle::INITIALIZING,
            Self::Ready => super::TextStyle::SUCCESS,
            Self::Warning => super::TextStyle::WARNING,
            Self::Error => super::TextStyle::ERROR,
        }
    }
}

pub struct StatusIndicator {
    position: Point,
    radius: u32,
    status: Status,
}

impl StatusIndicator {
    pub fn new(
        position: Point,
        radius: u32,
        status: Status,
    ) -> Self {
        Self {
            position,
            radius,
            status,
        }
    }


    pub fn status(&self) -> Status {
        self.status
    }

    pub fn draw(
        &self,
        framebuffer: &mut Framebuffer,
    ) {
        Circle::new(
            self.position,
            self.radius * 2,
        )
        .into_styled(
            PrimitiveStyle::with_fill(
                self.status.color(),
            ),
        )
        .draw(framebuffer)
        .unwrap();
    }
}