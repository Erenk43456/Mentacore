use mentacore_boot_protocol::BootInfo;

use crate::debug;

pub fn validate_boot_info(
    boot_info: *const BootInfo,
) -> &'static BootInfo {
    if boot_info.is_null() {
        debug::write(
            b"ERROR: BootInfo is NULL\r\n",
        );

        loop {
            core::hint::spin_loop();
        }
    }

    debug::write(
        b"BootInfo received.\r\n"
    );

    let boot_info =
        unsafe { &*boot_info };

    debug::write(
        b"BootInfo initialized.\r\n"
    );

    if boot_info.version
        != mentacore_boot_protocol::BOOT_PROTOCOL_VERSION
    {
        debug::write(
            b"ERROR: Unsupported BootInfo version.\r\n"
        );

        loop {
            core::hint::spin_loop();
        }
    }

    debug::write(
        b"Boot protocol version: "
    );

    debug::write_hex(
        boot_info.version as u64
    );

    debug::write(b"\r\n");

    debug::write(
        b"Userspace image address: "
    );

    debug::write_hex(
        boot_info.userspace_image_addr
    );

    debug::write(b"\r\n");

    debug::write(
        b"Userspace image size: "
    );

    debug::write_hex(
        boot_info.userspace_image_size
    );

    debug::write(b"\r\n");

    boot_info
}