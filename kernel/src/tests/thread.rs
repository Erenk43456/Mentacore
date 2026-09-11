use crate::thread::{
    Thread,
    ThreadState,
    KernelStack,
    KERNEL_STACK_SIZE,
};

pub(super) fn test_thread_creation() -> bool {
    let thread =
        Thread::new(1, 42);

    thread.tid() == 1
        && thread.process_id() == 42
        && thread.state() == ThreadState::Ready
}

pub(super) fn test_thread_state() -> bool {
    let mut thread =
        Thread::new(2, 42);

    if thread.state() != ThreadState::Ready {
        return false;
    }

    thread.set_state(ThreadState::Running);

    if thread.state() != ThreadState::Running {
        return false;
    }

    thread.set_state(ThreadState::Blocked);

    if thread.state() != ThreadState::Blocked {
        return false;
    }

    thread.set_state(ThreadState::Terminated);

    thread.state() == ThreadState::Terminated
}

pub(super) fn run(
    runner: &mut super::framework::TestRunner,
) {
    runner.run(
        b"thread::creation",
        test_thread_creation,
    );

    runner.run(
        b"thread::state",
        test_thread_state,
    );

    runner.run(
        b"thread::context",
        test_thread_context,
    );

    runner.run(
        b"thread::kernel_stack",
        test_kernel_stack,
    );

    runner.run(
        b"thread::kernel_stack_validation",
        test_kernel_stack_validation,
    );
}

pub(super) fn test_thread_context() -> bool {
    let thread =
        Thread::new(3, 42);

    let context =
        thread.context();

    context.rsp() == 0
        && context.rip() == 0
        && context.rflags() == 0x202
}

pub(super) fn test_kernel_stack() -> bool {
    let base = 0x0010_0000;
    let top = base + KERNEL_STACK_SIZE;

    let stack =
        match KernelStack::new(base, top) {
            Some(stack) => stack,
            None => return false,
        };

    stack.base() == base
        && stack.top() == top
        && stack.size() == KERNEL_STACK_SIZE
}

pub(super) fn test_kernel_stack_validation() -> bool {
    let base = 0x0010_0000;

    if KernelStack::new(
        base,
        base + KERNEL_STACK_SIZE - 1,
    ).is_some() {
        return false;
    }

    if KernelStack::new(
        base + 1,
        base + 1 + KERNEL_STACK_SIZE,
    ).is_some() {
        return false;
    }

    if KernelStack::new(
        base + KERNEL_STACK_SIZE,
        base,
    ).is_some() {
        return false;
    }

    true
}