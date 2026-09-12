mod ist;
mod state;
mod timer;
mod trap_frame;

use crate::hardware::lapic::Lapic;
use super::framework::TestRunner;

pub fn run(
    runner: &mut TestRunner,
    lapic: &Lapic,
) {
    runner.run(
        b"interrupts::state",
        state::test_interrupt_state,
    );

    runner.run(
        b"interrupts::TrapFrame",
        trap_frame::test_trap_frame_layout,
    );

    runner.run(
        b"interrupts::timer_stack_alignment",
        timer::test_timer_stack_alignment,
    );

    runner.run(
        b"interrupts::timer_stability",
        timer::test_timer_stability,
    );

    runner.run(
        b"interrupts::lapic_timer_stack_alignment",
        || timer::test_lapic_timer_stack_alignment(lapic),
    );

    runner.run(
        b"interrupts::lapic_timer_periodic",
        || timer::test_lapic_timer_periodic(lapic),
    );

    runner.run(
        b"interrupts::timer_context",
        || crate::interrupts::validate_interrupt_context_layout(),
    );

}

pub fn run_double_fault(
    runner: &mut TestRunner,
) {
    runner.run(
        b"interrupts::double_fault_ist1",
        ist::test_double_fault_ist1,
    );
}

pub fn record_timer_dispatch_rsp(
    rsp: u64,
) {
    timer::record_timer_dispatch_rsp(rsp);
}

pub fn record_lapic_timer_dispatch_rsp(
    rsp: u64,
) {
    timer::record_lapic_timer_dispatch_rsp(rsp);
}