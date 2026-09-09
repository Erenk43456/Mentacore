use crate::cpu;
use crate::debug;

pub fn initialize() {
    debug::write(b"Initializing CPU state...\r\n");

    cpu::init();

    debug::write(b"CPU state initialized.\r\n");
}