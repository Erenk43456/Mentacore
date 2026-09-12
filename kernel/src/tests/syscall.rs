use crate::memory::paging::{
    KERNEL_SPACE_START,
    USER_SPACE_END,
};
use crate::scheduler::SchedulerRuntime;
use crate::syscall::{
    dispatch,
    validate_user_address,
    SyscallContext,
    SYS_GET_TID,
    SYSCALL_ENOSYS,
};

use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"syscall::context_registers",
        test_context_registers,
    );

    runner.run(
        b"syscall::dispatch_get_tid",
        test_get_tid,
    );

    runner.run(
        b"syscall::dispatch_unknown",
        test_unknown_syscall,
    );

    runner.run(
        b"syscall::user_address_validation",
        test_user_address_validation,
    );
}

fn test_context_registers() -> bool {
    let context = SyscallContext::new(
        0x10,
        0x20,
        0x30,
        0x40,
        0x50,
        0x60,
        0x70,
    );

    context.number() == 0x10
        && context.arg0() == 0x20
        && context.arg1() == 0x30
        && context.arg2() == 0x40
        && context.arg3() == 0x50
        && context.arg4() == 0x60
        && context.arg5() == 0x70
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

    dispatch(&context) == SYSCALL_ENOSYS
}

fn test_user_address_validation() -> bool {
    let user_address = USER_SPACE_END;
    let kernel_address = KERNEL_SPACE_START;
    let non_canonical_address = 0x0000_8000_0000_0000;

    validate_user_address(user_address)
        && !validate_user_address(kernel_address)
        && !validate_user_address(non_canonical_address)
}