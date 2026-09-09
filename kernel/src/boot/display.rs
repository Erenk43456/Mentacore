use crate::debug;
use crate::display::{boot_ui, Framebuffer};
use crate::boot_state::BootState;
use mentacore_boot_protocol::BootInfo;

pub unsafe fn initialize(
    boot_info: &BootInfo,
) {
    debug::write(
        b"Initializing display renderer...\r\n"
    );

    let mut framebuffer = unsafe {
        Framebuffer::from_boot_info(boot_info)
    };

    debug::write(
        b"Display renderer initialized.\r\n"
    );

    let state =
        BootState::from_boot_info(boot_info);

    debug::write(
        b"Rendering Mentacore boot UI...\r\n"
    );

    boot_ui::render(
        &mut framebuffer,
        state,
    );

    debug::write(
        b"DISPLAY OK\r\n"
    );
}