use core::sync::atomic::{AtomicU64, Ordering};

use crate::cpu;
use crate::debug;

use super::framework::TestRunner;

static TIMER_DISPATCH_RSP: AtomicU64 =
    AtomicU64::new(0);

static LAPIC_TIMER_DISPATCH_RSP: AtomicU64 =
    AtomicU64::new(0);

pub fn run(
    runner: &mut TestRunner,
    lapic: &crate::hardware::lapic::Lapic,
) {
    runner.run(
        b"interrupts::state",
        test_interrupt_state,
    );

    runner.run(
        b"interrupts::TrapFrame",
        test_trap_frame_layout,
    );

    runner.run(
        b"interrupts::timer_stack_alignment",
        test_timer_stack_alignment,
    );

    runner.run(
        b"interrupts::timer_stability",
        test_timer_stability,
    );

    runner.run(
        b"interrupts::lapic_timer_stack_alignment",
        || test_lapic_timer_stack_alignment(lapic),
    );

    runner.run(
        b"interrupts::double_fault_ist1",
        test_double_fault_ist1,
    );
}

pub fn record_timer_dispatch_rsp(rsp: u64) {
    TIMER_DISPATCH_RSP.store(
        rsp,
        Ordering::Relaxed,
    );
}

pub fn record_lapic_timer_dispatch_rsp(
    rsp: u64,
) {
    LAPIC_TIMER_DISPATCH_RSP.store(
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

fn test_trap_frame_layout() -> bool {
    use crate::interrupts::TrapFrame;

    if core::mem::size_of::<TrapFrame>()
        != 13 * core::mem::size_of::<u64>()
    {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r11) != 0 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r10) != 8 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r9) != 16 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r8) != 24 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rdi) != 32 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rsi) != 40 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rdx) != 48 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rcx) != 56 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rax) != 64 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, error_code) != 72 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rip) != 80 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, cs) != 88 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rflags) != 96 {
        return false;
    }

    true
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

fn test_timer_stability() -> bool {
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

fn test_lapic_timer_stack_alignment(
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

    // Wait until the LAPIC timer interrupt has
    // reached the Rust dispatch function.
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

    // SysV x86-64 ABI:
    // RSP must be 16-byte aligned immediately
    // before the CALL instruction.
    alignment == 0
}

fn test_double_fault_ist1() -> bool {
    crate::interrupts::trigger_double_fault_test()
}