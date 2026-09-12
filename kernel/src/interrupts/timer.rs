use core::sync::atomic::{AtomicU64, Ordering};

use crate::thread::InterruptContext;

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

    /*
     * The LAPIC interrupt must be acknowledged before
     * transferring control to another thread.
     */
    unsafe {
        crate::hardware::lapic::write_global_eoi();
    }

    let next_rsp =
        match crate::scheduler::SchedulerRuntime::preempt(
            dispatch_rsp,
        ) {
            Some(rsp) => rsp,
            None => return,
        };

    /*
     * No runnable thread change occurred.
     * Continue through the normal interrupt return path.
     */
    if next_rsp == dispatch_rsp {
        return;
    }

    /*
     * interrupt_context_switch() stores the current
     * interrupt-entry RSP before loading the next
     * thread's pre-built InterruptContext.
     *
     * It never returns; iretq resumes the selected thread.
     */
    let mut current_rsp = dispatch_rsp;

    unsafe {
        crate::thread::interrupt_context_switch(
            &raw mut current_rsp,
            next_rsp as *const crate::thread::InterruptContext,
        );
    }
}

#[cfg(feature = "kernel-tests")]
pub fn validate_interrupt_context_layout() -> bool {
    core::mem::size_of::<InterruptContext>() == 20 * 8
}