use core::sync::atomic::{AtomicU64, Ordering};

pub const LAPIC_TIMER_VECTOR: u8 = 0x40;

static TIMER_TICKS: AtomicU64 =
    AtomicU64::new(0);

#[cfg(feature = "kernel-tests")]
static LAPIC_TIMER_TICKS: AtomicU64 =
    AtomicU64::new(0);

pub fn timer_ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed)
}

#[cfg(feature = "kernel-tests")]
pub fn lapic_timer_ticks() -> u64 {
    LAPIC_TIMER_TICKS.load(Ordering::Relaxed)
}

#[unsafe(no_mangle)]
extern "C" fn timer_irq_dispatch(
    dispatch_rsp: u64,
) {
    #[cfg(feature = "kernel-tests")]
    crate::tests::interrupts::record_timer_dispatch_rsp(
        dispatch_rsp,
    );

    #[cfg(not(feature = "kernel-tests"))]
    let _ = dispatch_rsp;

    TIMER_TICKS.fetch_add(
        1,
        Ordering::Relaxed,
    );

    unsafe {
        crate::hardware::pic::send_eoi(0);
    }
}

#[unsafe(no_mangle)]
extern "C" fn lapic_timer_dispatch(
    dispatch_rsp: u64,
) {
    #[cfg(feature = "kernel-tests")]
    {
        crate::tests::interrupts::record_lapic_timer_dispatch_rsp(
            dispatch_rsp,
        );

        LAPIC_TIMER_TICKS.fetch_add(
            1,
            Ordering::Relaxed,
        );
    }

    #[cfg(not(feature = "kernel-tests"))]
    let _ = dispatch_rsp;

    unsafe {
        crate::hardware::lapic::write_global_eoi();
    }
}