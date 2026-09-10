use crate::cpu;

pub(super) fn test_interrupt_state() -> bool {
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

    let enabled_state =
        cpu::InterruptState::save_and_disable();

    if !enabled_state.were_enabled() {
        return false;
    }

    if cpu::interrupts_enabled() {
        return false;
    }

    enabled_state.restore();

    if !cpu::interrupts_enabled() {
        return false;
    }

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

    unsafe {
        core::arch::asm!(
            "sti",
            options(nostack)
        );
    }

    cpu::interrupts_enabled()
}