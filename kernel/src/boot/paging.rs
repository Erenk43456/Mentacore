use mentacore_boot_protocol::BootInfo;

use crate::debug;
use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

pub unsafe fn initialize(
    allocator: &mut PhysicalFrameAllocator,
    boot_info: &BootInfo,
) {
    debug::write(b"Allocated frames before paging: ");
    debug::write_hex(allocator.allocated_count());
    debug::write(b"\r\n");

    debug::write(b"Initializing paging...\r\n");

    match unsafe {
        memory::paging::init(
            allocator,
            boot_info,
        )
    } {
        Ok(()) => {
            debug::write(
                b"Paging initialized.\r\n"
            );

            debug::write(
                b"Allocated frames after paging: "
            );
            debug::write_hex(
                allocator.allocated_count()
            );
            debug::write(b"\r\n");
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