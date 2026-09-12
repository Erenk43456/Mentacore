use crate::scheduler::SchedulerRuntime;
use crate::syscall::{
    dispatch,
    SyscallContext,
    SYS_GET_TID,
};

use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"syscall::dispatch_get_tid",
        test_get_tid,
    );

    runner.run(
        b"syscall::dispatch_unknown",
        test_unknown_syscall,
    );
}

fn test_get_tid() -> bool {
    let expected = SchedulerRuntime::current_thread_id();

    let context = SyscallContext::new(
        SYS_GET_TID,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let result = dispatch(&context);

    expected
        .map(|thread_id| result == thread_id as u64)
        .unwrap_or(false)
}

fn test_unknown_syscall() -> bool {
    let context = SyscallContext::new(
        0xFFFF_FFFF_FFFF_FFFEu64,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    dispatch(&context) == u64::MAX
}