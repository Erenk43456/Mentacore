use crate::cpu;

use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"interrupts::state",
        test_interrupt_state,
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