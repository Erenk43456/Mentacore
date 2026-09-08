use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    text::Text,
};

use super::{
    super::Framebuffer,
    Status,
    TextLabel,
    TextStyle,
};

pub struct InfoRow {
    position: Point,
    label: &'static str,
    value: Value,
    value_style: TextStyle,
}

enum Value {
    Text(&'static str),
    Resolution(u32, u32),
}

impl InfoRow {
    pub fn new(
        position: Point,
        label: &'static str,
        value: &'static str,
        value_style: TextStyle,
    ) -> Self {
        Self {
            position,
            label,
            value: Value::Text(value),
            value_style,
        }
    }

    pub fn resolution(
        position: Point,
        label: &'static str,
        width: u32,
        height: u32,
        value_style: TextStyle,
    ) -> Self {
        Self {
            position,
            label,
            value: Value::Resolution(width, height),
            value_style,
        }
    }

    pub fn status(
        position: Point,
        label: &'static str,
        status: Status,
    ) -> Self {
        Self::new(
            position,
            label,
            status.text(),
            status.text_style(),
        )
    }

    pub fn draw(
        &self,
        framebuffer: &mut Framebuffer,
    ) {
        let label = TextLabel::new(
            self.position,
            self.label,
            TextStyle::BODY,
        );

        label.draw(framebuffer);

        let value_position = Point::new(
            self.position.x + 300,
            self.position.y,
        );

        match self.value {
            Value::Text(text) => {
                let value = TextLabel::new(
                    value_position,
                    text,
                    self.value_style,
                );

                value.draw(framebuffer);
            }

            Value::Resolution(width, height) => {
                let text_style = MonoTextStyle::new(
                    self.value_style.font,
                    self.value_style.color,
                );

                Text::new(
                    "",
                    value_position,
                    text_style,
                )
                .draw(framebuffer)
                .unwrap();

                draw_number(
                    framebuffer,
                    value_position,
                    width,
                    self.value_style,
                );

                let separator = TextLabel::new(
                    Point::new(
                        value_position.x + 42,
                        value_position.y,
                    ),
                    " X ",
                    self.value_style,
                );

                separator.draw(framebuffer);

                draw_number(
                    framebuffer,
                    Point::new(
                        value_position.x + 66,
                        value_position.y,
                    ),
                    height,
                    self.value_style,
                );
            }
        }
    }
}

fn draw_number(
    framebuffer: &mut Framebuffer,
    position: Point,
    value: u32,
    style: TextStyle,
) {
    let mut buffer = [0u8; 10];
    let mut value = value;
    let mut index = buffer.len();

    if value == 0 {
        buffer[index - 1] = b'0';
        index -= 1;
    } else {
        while value > 0 {
            index -= 1;
            buffer[index] = b'0' + (value % 10) as u8;
            value /= 10;
        }
    }

    let text_style = MonoTextStyle::new(
        style.font,
        style.color,
    );

    Text::new(
        core::str::from_utf8(&buffer[index..])
            .unwrap(),
        position,
        text_style,
    )
    .draw(framebuffer)
    .unwrap();
}