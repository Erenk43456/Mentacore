use core::sync::atomic::{
    AtomicU64,
    Ordering,
};

use crate::debug;

static TIMER_DISPATCH_RSP: AtomicU64 =
    AtomicU64::new(0);

static LAPIC_TIMER_DISPATCH_RSP: AtomicU64 =
    AtomicU64::new(0);

pub(super) fn record_timer_dispatch_rsp(
    rsp: u64,
) {
    TIMER_DISPATCH_RSP.store(
        rsp,
        Ordering::Relaxed,
    );
}

pub(super) fn record_lapic_timer_dispatch_rsp(
    rsp: u64,
) {
    LAPIC_TIMER_DISPATCH_RSP.store(
        rsp,
        Ordering::Relaxed,
    );
}

pub(super) fn test_timer_stack_alignment() -> bool {
    TIMER_DISPATCH_RSP.store(
        0,
        Ordering::Relaxed,
    );

    for _ in 0..10_000_000 {
        if TIMER_DISPATCH_RSP.load(
            Ordering::Relaxed,
        ) != 0 {
            break;
        }

        core::hint::spin_loop();
    }

    let rsp =
        TIMER_DISPATCH_RSP.load(
            Ordering::Relaxed,
        );

    if rsp == 0 {
        return false;
    }

    debug::write(b"\r\n[TIMER ABI] RSP = ");
    debug::write_hex(rsp);
    debug::write(b"\r\n");

    debug::write(
        b"[TIMER ABI] Call-site RSP % 16 = ",
    );

    let alignment = rsp & 0xF;

    debug::write_hex(alignment);
    debug::write(b"\r\n");

    alignment == 0
}

pub(super) fn test_timer_stability() -> bool {
    let mut previous_ticks =
        crate::interrupts::timer_ticks();

    for _ in 0..8 {
        let target_ticks =
            previous_ticks + 1;

        let mut advanced = false;

        for _ in 0..10_000_000 {
            let current_ticks =
                crate::interrupts::timer_ticks();

            if current_ticks >= target_ticks {
                debug::write(
                    b"\r\n[TIMER STABILITY] tick = ",
                );
                debug::write_hex(current_ticks);
                debug::write(b"\r\n");

                if current_ticks <= previous_ticks {
                    return false;
                }

                previous_ticks = current_ticks;
                advanced = true;
                break;
            }

            core::hint::spin_loop();
        }

        if !advanced {
            debug::write(
                b"\r\n[TIMER STABILITY] timeout, ticks = ",
            );
            debug::write_hex(previous_ticks);
            debug::write(b"\r\n");

            return false;
        }
    }

    true
}

pub(super) fn test_lapic_timer_stack_alignment(
    lapic: &crate::hardware::lapic::Lapic,
) -> bool {
    unsafe {
        lapic.arm_timer_oneshot(
            crate::interrupts::LAPIC_TIMER_VECTOR,
            100_000_000,
        );
    }

    LAPIC_TIMER_DISPATCH_RSP.store(
        0,
        Ordering::Relaxed,
    );

    for _ in 0..10_000_000 {
        if LAPIC_TIMER_DISPATCH_RSP.load(
            Ordering::Relaxed,
        ) != 0 {
            break;
        }

        core::hint::spin_loop();
    }

    let rsp =
        LAPIC_TIMER_DISPATCH_RSP.load(
            Ordering::Relaxed,
        );

    if rsp == 0 {
        return false;
    }

    debug::write(
        b"\r\n[LAPIC TIMER ABI] RSP = ",
    );
    debug::write_hex(rsp);
    debug::write(b"\r\n");

    debug::write(
        b"[LAPIC TIMER ABI] Call-site RSP % 16 = ",
    );

    let alignment = rsp & 0xF;

    debug::write_hex(alignment);
    debug::write(b"\r\n");

    alignment == 0
}