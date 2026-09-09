use crate::debug;
use crate::interrupts;
use crate::memory::physical::PhysicalFrameAllocator;

pub unsafe fn initialize(
    allocator: PhysicalFrameAllocator,
) {
    unsafe {
        interrupts::init(allocator);
    }

    debug::write(
        b"Interrupt system initialized.\r\n"
    );
}