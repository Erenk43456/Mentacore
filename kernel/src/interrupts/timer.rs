use core::sync::atomic::{AtomicU64, Ordering};

use crate::thread::InterruptContext;

pub const LAPIC_TIMER_VECTOR: u8 = 0x40;

static TIMER_TICKS: AtomicU64 =
    AtomicU64::new(0);

#[cfg(feature = "kernel-tests")]
static LAPIC_TIMER_TICKS: AtomicU64 =
    AtomicU64::new(0);

#[cfg(feature = "kernel-tests")]
static LAPIC_TIMER_PREEMPTION_ENABLED: AtomicU64 =
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

    unsafe {
        crate::hardware::lapic::write_global_eoi();
    }

    #[cfg(feature = "kernel-tests")]
    if LAPIC_TIMER_PREEMPTION_ENABLED.load(
        Ordering::Relaxed,
    ) == 0
    {
        return;
    }

    // A LAPIC timer interrupt can arrive while executing either
    // kernel code (CPL0) or user code (CPL3). Only preempt a
    // user-mode frame here; kernel-thread bootstrap frames are
    // managed separately and must not be mixed with a Ring 3 frame.
    let cs = unsafe {
        *((dispatch_rsp + 16 * 8) as *const u64)
    };

    if (cs & 0x3) != 0x3 {
        return;
    }

    let (next_rsp, next_pml4) =
        match crate::scheduler::SchedulerRuntime::preempt(
            dispatch_rsp,
        ) {
            Some(value) => value,
            None => return,
        };

    if next_rsp == dispatch_rsp {
        return;
    }

    let mut current_rsp = dispatch_rsp;

    unsafe {
        crate::thread::interrupt_context_switch_to_address_space(
            &raw mut current_rsp,
            next_rsp as *const u64,
            next_pml4,
        );
    }
}

#[cfg(feature = "kernel-tests")]
pub fn validate_interrupt_context_layout() -> bool {
    core::mem::size_of::<InterruptContext>() == 20 * 8
}

#[cfg(feature = "kernel-tests")]
pub fn enable_lapic_timer_preemption() {
    LAPIC_TIMER_PREEMPTION_ENABLED.store(
        1,
        Ordering::Relaxed,
    );
}

#[cfg(feature = "kernel-tests")]
pub fn disable_lapic_timer_preemption() {
    LAPIC_TIMER_PREEMPTION_ENABLED.store(
        0,
        Ordering::Relaxed,
    );
}