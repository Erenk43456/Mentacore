pub(crate) fn serial_write(message: &[u8]) {
    crate::debug::write(message);
}

pub(crate) fn serial_write_hex(value: u64) {
    crate::debug::write_hex(value);
}