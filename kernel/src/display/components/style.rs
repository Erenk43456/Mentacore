use embedded_graphics::{
    mono_font::{
        ascii::{
            FONT_6X10,
            FONT_8X13,
            FONT_10X20,
        },
        MonoFont,
    },
    pixelcolor::Rgb888,
};

use super::super::color;

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub font: &'static MonoFont<'static>,
    pub color: Rgb888,
}

impl TextStyle {
    pub const TITLE: Self = Self {
        font: &FONT_10X20,
        color: color::TEXT,
    };

    pub const BODY: Self = Self {
        font: &FONT_8X13,
        color: color::TEXT,
    };

    pub const MUTED: Self = Self {
        font: &FONT_8X13,
        color: color::TEXT_MUTED,
    };

    pub const INITIALIZING: Self = Self {
        font: &FONT_8X13,
        color: color::ACCENT,
    };

    pub const SUCCESS: Self = Self {
        font: &FONT_8X13,
        color: color::SUCCESS,
    };

    pub const WARNING: Self = Self {
        font: &FONT_8X13,
        color: color::WARNING,
    };

    pub const ERROR: Self = Self {
        font: &FONT_8X13,
        color: color::ERROR,
    };

    pub const SMALL: Self = Self {
        font: &FONT_6X10,
        color: color::TEXT_MUTED,
    };
}