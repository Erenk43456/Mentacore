use crate::debug;
use crate::memory;

pub unsafe fn initialize() {
    memory::heap::init();

    debug::write(
        b"Heap initialized.\r\n"
    );
}