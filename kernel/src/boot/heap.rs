use crate::debug;
use crate::memory;

pub unsafe fn initialize() {
    debug::write(
        b"Initializing kernel heap...\r\n"
    );

    unsafe {
        memory::heap::init();
    }

    debug::write(
        b"Kernel heap initialized.\r\n"
    );
}