use embedded_graphics::{
    prelude::*,
    primitives::{
        PrimitiveStyle,
        Rectangle,
    },
};

use super::{
    color,
    components::{
        Header,
        InfoRow,
        Status,
        StatusIndicator,
        TextLabel,
        TextStyle,
    },
    geometry::BootLayout,
    Framebuffer,
};

const CONTENT_WIDTH: u32 = 560;
const INFO_WIDTH: u32 = 440;

pub fn render(framebuffer: &mut Framebuffer) {
    let layout = BootLayout::new(framebuffer);
    let screen = layout.screen();

    framebuffer
        .clear(color::BACKGROUND)
        .unwrap();

    let center_x = screen.center_x();

    // Header
    let header = Header::new(
        Point::new(
            screen.centered_x(150),
            layout.header_y(),
        ),
        "MENTACORE",
    );

    header.draw(framebuffer);

    let subtitle = TextLabel::new(
        Point::new(
            screen.centered_x(164),
            layout.subtitle_y(),
        ),
        "OPERATING ENVIRONMENT",
        TextStyle::SMALL,
    );

    subtitle.draw(framebuffer);

    // Header separator
    Rectangle::new(
        Point::new(
            screen.centered_x(CONTENT_WIDTH),
            layout.header_separator_y(),
        ),
        Size::new(CONTENT_WIDTH, 1),
    )
    .into_styled(
        PrimitiveStyle::with_fill(color::BORDER),
    )
    .draw(framebuffer)
    .unwrap();

    // System status
    let status = StatusIndicator::new(
        Point::new(
            center_x - 110,
            layout.status_y(),
        ),
        6,
        Status::Ready,
    );

    status.draw(framebuffer);

    let status_text = TextLabel::new(
        Point::new(
            center_x - 85,
            layout.status_text_y(),
        ),
        status.status().text(),
        TextStyle::SUCCESS,
    );

    status_text.draw(framebuffer);

    // System information
    let info_x = screen.centered_x(INFO_WIDTH);

    InfoRow::ready(
        Point::new(info_x, layout.info_y(0)),
        "Kernel",
    )
    .draw(framebuffer);

    InfoRow::ready(
        Point::new(info_x, layout.info_y(1)),
        "Bootloader",
    )
    .draw(framebuffer);

    InfoRow::ready(
        Point::new(info_x, layout.info_y(2)),
        "Graphics",
    )
    .draw(framebuffer);

    let (width, height) = framebuffer.resolution();

    InfoRow::resolution(
        Point::new(info_x, layout.info_y(3)),
        "Framebuffer",
        width,
        height,
        TextStyle::MUTED,
    )
    .draw(framebuffer);

    // Footer separator
    Rectangle::new(
        Point::new(
            screen.centered_x(INFO_WIDTH),
            layout.footer_separator_y(),
        ),
        Size::new(INFO_WIDTH, 1),
    )
    .into_styled(
        PrimitiveStyle::with_fill(color::BORDER),
    )
    .draw(framebuffer)
    .unwrap();

    // Footer
    let footer = TextLabel::new(
        Point::new(
            screen.centered_x(180),
            layout.footer_y(),
        ),
        "INITIALIZING SYSTEM",
        TextStyle::SMALL,
    );

    footer.draw(framebuffer);
}