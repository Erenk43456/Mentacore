use crate::cpu;
use crate::debug;

pub fn initialize() {
    cpu::init();

    debug::write(
        b"CPU initialized.\r\n"
    );
}