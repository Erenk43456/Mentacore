use mentacore_boot_protocol::BootInfo;

use crate::debug;
use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

pub unsafe fn initialize(
    frame_allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) {
    match memory::paging::init(
        frame_allocator,
        boot_info,
    ) {
        Ok(()) => {
            debug::write(
                b"Paging initialized.\r\n"
            );
        }

        Err(()) => {
            debug::write(
                b"ERROR: Failed to initialize paging\r\n"
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }
}