use crate::debug;
use crate::interrupts;
use crate::memory::physical::PhysicalFrameAllocator;

pub unsafe fn initialize(
    frame_allocator: PhysicalFrameAllocator,
) {
    unsafe {
        interrupts::init(frame_allocator);
    }

    debug::write(
        b"Interrupt system initialized.\r\n"
    );
}