use crate::cpu;
use crate::sync::Spinlock;

use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"sync::spinlock",
        test_spinlock,
    );

    runner.run(
        b"sync::spinlock_irqsave",
        test_spinlock_irqsave,
    );
}

fn test_spinlock() -> bool {
    let lock = Spinlock::new(0u64);

    // Normal lock acquisition must not alter interrupt state.
    if !cpu::interrupts_enabled() {
        return false;
    }

    if lock.is_locked() {
        return false;
    }

    {
        let mut guard = lock.lock();

        if !cpu::interrupts_enabled() {
            return false;
        }

        if !lock.is_locked() {
            return false;
        }

        *guard = 0x1234_5678;
    }

    // Guard drop must release the lock and preserve interrupts.
    if lock.is_locked() {
        return false;
    }

    if !cpu::interrupts_enabled() {
        return false;
    }

    // Lock must be reusable and protected data must persist.
    {
        let mut guard = lock.lock();

        if !cpu::interrupts_enabled() {
            return false;
        }

        if *guard != 0x1234_5678 {
            return false;
        }

        *guard = 0xCAFE_BABE;
    }

    if lock.is_locked() {
        return false;
    }

    if !cpu::interrupts_enabled() {
        return false;
    }

    // Final acquisition verifies the updated value survived another cycle.
    {
        let guard = lock.lock();

        if *guard != 0xCAFE_BABE {
            return false;
        }
    }

    !lock.is_locked()
}

fn test_spinlock_irqsave() -> bool {
    let lock = Spinlock::new(0u64);

    // 1. Interrupts initially enabled.
    if !cpu::interrupts_enabled() {
        return false;
    }

    {
        let mut guard = lock.lock_irqsave();

        if cpu::interrupts_enabled() {
            return false;
        }

        *guard = 0xDEAD_BEEF;
    }

    // Original interrupt state must be restored.
    if !cpu::interrupts_enabled() {
        return false;
    }

    // 2. Protected data must survive unlock.
    {
        let guard = lock.lock_irqsave();

        if *guard != 0xDEAD_BEEF {
            return false;
        }
    }

    // 3. Acquire while interrupts are already disabled.
    let disabled_state =
        cpu::InterruptState::save_and_disable();

    if cpu::interrupts_enabled() {
        return false;
    }

    {
        let mut guard = lock.lock_irqsave();

        if cpu::interrupts_enabled() {
            disabled_state.restore();
            return false;
        }

        *guard = 0xCAFE_BABE;
    }

    // lock_irqsave() must preserve the disabled state.
    if cpu::interrupts_enabled() {
        disabled_state.restore();
        return false;
    }

    disabled_state.restore();

    if !cpu::interrupts_enabled() {
        return false;
    }

    true
}