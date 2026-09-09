use crate::debug;
use crate::hardware;

pub unsafe fn initialize() {
    unsafe {
        hardware::pic::remap();
        hardware::pic::enable_irq(0);
    }

    debug::write(
        b"PIC initialized.\r\n"
    );

    unsafe {
        hardware::pit::set_frequency(100);
    }

    debug::write(
        b"PIT initialized.\r\n"
    );
}