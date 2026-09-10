use crate::debug;

use core::sync::atomic::{
    AtomicUsize,
    Ordering,
};

static PASSED: AtomicUsize =
    AtomicUsize::new(0);

static TOTAL: AtomicUsize =
    AtomicUsize::new(0);

pub struct TestRunner {
    passed: usize,
    total: usize,
}

impl TestRunner {
    pub const fn new() -> Self {
        Self {
            passed: 0,
            total: 0,
        }
    }

    pub fn run(
        &mut self,
        name: &'static [u8],
        test: impl FnOnce() -> bool,
    ) {
        self.total += 1;

        TOTAL.store(
            self.total,
            Ordering::Relaxed,
        );

        debug::write(b"[TEST] ");
        debug::write(name);
        debug::write(b" ... ");

        if test() {
            self.passed += 1;

            PASSED.store(
                self.passed,
                Ordering::Relaxed,
            );

            debug::write(b"OK\r\n");
        } else {
            debug::write(b"FAILED\r\n");
        }
    }

    pub fn finish(&self) {
        debug::write(b"\r\nRESULT: ");
        write_usize(self.passed);
        debug::write(b"/");
        write_usize(self.total);
        debug::write(b" TESTS PASSED\r\n");

        if self.passed == self.total {
            debug::write(b"ALL TESTS PASSED\r\n");
        } else {
            debug::write(b"TESTS FAILED\r\n");
        }
    }
}

pub fn result() -> (usize, usize) {
    (
        PASSED.load(Ordering::Relaxed),
        TOTAL.load(Ordering::Relaxed),
    )
}

pub fn write_header() {
    debug::write(
        b"\r\n================================\r\n",
    );
    debug::write(
        b"       MENTACORE KERNEL TESTS\r\n",
    );
    debug::write(
        b"================================\r\n\r\n",
    );
}

pub fn write_usize(value: usize) {
    if value == 0 {
        debug::write(b"0");
        return;
    }

    let mut buffer = [0u8; 20];
    let mut index = buffer.len();
    let mut value = value;

    while value != 0 {
        index -= 1;
        buffer[index] =
            b'0' + (value % 10) as u8;
        value /= 10;
    }

    debug::write(&buffer[index..]);
}