use crate::boot_state::BootState;
use crate::debug;
use crate::display::{boot_ui, Framebuffer};
use mentacore_boot_protocol::BootInfo;

pub unsafe fn initialize(
    boot_info: &BootInfo,
) {
    let mut framebuffer = unsafe {
        Framebuffer::from_boot_info(boot_info)
    };

    let state =
        BootState::from_boot_info(boot_info);

    boot_ui::render(
        &mut framebuffer,
        state,
    );

    debug::write(
        b"Display initialized.\r\n"
    );
}