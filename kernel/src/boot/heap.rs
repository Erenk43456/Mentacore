use crate::debug;
use crate::memory;

pub unsafe fn initialize() {
    unsafe {
        memory::heap::init();
    }

    debug::write(
        b"Heap initialized.\r\n"
    );
}