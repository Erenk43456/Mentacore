use core::sync::atomic::{AtomicU64, Ordering};

use crate::cpu;
use crate::debug;

use super::framework::TestRunner;

static TIMER_DISPATCH_RSP: AtomicU64 =
    AtomicU64::new(0);

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"interrupts::state",
        test_interrupt_state,
    );

    runner.run(
        b"interrupts::timer_stack_alignment",
        test_timer_stack_alignment,
    );
}

pub fn record_timer_dispatch_rsp(rsp: u64) {
    TIMER_DISPATCH_RSP.store(
        rsp,
        Ordering::Relaxed,
    );
}

fn test_interrupt_state() -> bool {
    // At this point the interrupt system must
    // already be active.
    if !cpu::interrupts_enabled() {
        unsafe {
            core::arch::asm!(
                "sti",
                options(nostack)
            );
        }
    }

    if !cpu::interrupts_enabled() {
        return false;
    }

    // Verify that save_and_disable() captures
    // the enabled state and disables interrupts.
    let enabled_state =
        cpu::InterruptState::save_and_disable();

    if !enabled_state.were_enabled() {
        return false;
    }

    if cpu::interrupts_enabled() {
        return false;
    }

    // Restore the state that existed before
    // save_and_disable().
    enabled_state.restore();

    if !cpu::interrupts_enabled() {
        return false;
    }

    // Verify that an already-disabled state
    // remains disabled.
    unsafe {
        core::arch::asm!(
            "cli",
            options(nostack)
        );
    }

    if cpu::interrupts_enabled() {
        return false;
    }

    let disabled_state =
        cpu::InterruptState::save_and_disable();

    if disabled_state.were_enabled() {
        return false;
    }

    disabled_state.restore();

    if cpu::interrupts_enabled() {
        return false;
    }

    // Leave the system in the normal
    // interrupt-enabled state.
    unsafe {
        core::arch::asm!(
            "sti",
            options(nostack)
        );
    }

    cpu::interrupts_enabled()
}

fn test_timer_stack_alignment() -> bool {
    TIMER_DISPATCH_RSP.store(
        0,
        Ordering::Relaxed,
    );

    // Wait until the PIT timer interrupt has
    // reached the Rust dispatch function.
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

    debug::write(b"[TIMER ABI] Call-site RSP % 16 = ");

    let alignment = rsp & 0xF;

    debug::write_hex(alignment);
    debug::write(b"\r\n");

    // SysV x86-64 ABI:
    // RSP must be 16-byte aligned immediately
    // before the CALL instruction.
    alignment == 0
}