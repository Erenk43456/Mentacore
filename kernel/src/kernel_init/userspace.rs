use core::slice;

use mentacore_boot_protocol::BootInfo;

use crate::{
    cpu,
    debug,
    memory,
    memory::physical::PhysicalFrameAllocator,
};

pub unsafe fn load(
    boot_info: &BootInfo,
    frame_allocator: &mut PhysicalFrameAllocator,
) -> memory::LoadedElf {
    let userspace_image_size =
        match usize::try_from(
            boot_info.userspace_image_size,
        ) {
            Ok(size) => size,

            Err(_) => {
                debug::write(
                    b"ERROR: Userspace ELF size overflow.\r\n"
                );

                loop {
                    cpu::halt();
                }
            }
        };

    let userspace_image_end =
        match boot_info
            .userspace_image_addr
            .checked_add(
                boot_info.userspace_image_size,
            ) {
            Some(end) => end,

            None => {
                debug::write(
                    b"ERROR: Userspace ELF address overflow.\r\n"
                );

                loop {
                    cpu::halt();
                }
            }
        };

    if userspace_image_end
        > memory::paging::IDENTITY_MAP_SIZE
    {
        debug::write(
            b"ERROR: Userspace ELF is outside identity map.\r\n"
        );

        loop {
            cpu::halt();
        }
    }

    let userspace_image =
        unsafe {
            slice::from_raw_parts(
                boot_info.userspace_image_addr
                    as *const u8,
                userspace_image_size,
            )
        };

    let loaded_userspace =
        match unsafe {
            memory::load_elf(
                userspace_image,
                frame_allocator,
            )
        } {
            Ok(loaded) => loaded,

            Err(_) => {
                debug::write(
                    b"ERROR: Failed to load userspace ELF.\r\n"
                );

                loop {
                    cpu::halt();
                }
            }
        };

    debug::write(
        b"Userspace ELF loaded into address space.\r\n"
    );

    debug::write(
        b"Userspace entry: "
    );

    debug::write_hex(
        loaded_userspace.entry()
    );

    debug::write(b"\r\n");

    debug::write(
        b"Userspace segments: "
    );

    debug::write_hex(
        loaded_userspace.segment_count()
            as u64
    );

    debug::write(b"\r\n");

    loaded_userspace
}