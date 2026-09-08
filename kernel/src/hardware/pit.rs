use core::arch::asm;

const PIT_COMMAND: u16 = 0x43;
const PIT_CHANNEL0: u16 = 0x40;

const PIT_BASE_FREQUENCY: u32 = 1_193_182;

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nostack, preserves_flags)
        );
    }
}

pub unsafe fn set_frequency(frequency: u32) {
    if frequency == 0 {
        return;
    }

    let divisor =
        PIT_BASE_FREQUENCY / frequency;

    if divisor == 0 || divisor > 0xFFFF {
        return;
    }

    unsafe {
        // Channel 0, access mode lobyte/hibyte,
        // mode 2 (rate generator), binary mode.
        outb(
            PIT_COMMAND,
            0x34,
        );

        outb(
            PIT_CHANNEL0,
            (divisor & 0xFF) as u8,
        );

        outb(
            PIT_CHANNEL0,
            ((divisor >> 8) & 0xFF) as u8,
        );
    }
}