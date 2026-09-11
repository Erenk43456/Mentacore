use crate::memory::physical::PhysicalFrameAllocator;
use crate::thread::{
    KernelStack,
    KERNEL_STACK_ALIGNMENT,
    KERNEL_STACK_PAGES,
    KERNEL_STACK_SIZE,
    Thread,
    ThreadState,
};

fn test_stack() -> KernelStack {
    KernelStack::new(
        0x0010_0000,
        0x0010_0000 + KERNEL_STACK_SIZE,
    )
    .unwrap()
}

pub(super) fn test_thread_creation(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let thread =
        match Thread::new(
            1,
            42,
            allocator,
        ) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

    thread.tid() == 1
        && thread.process_id() == 42
        && thread.state() == ThreadState::Ready
}

pub(super) fn test_thread_state(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut thread =
        match Thread::new(
            2,
            42,
            allocator,
        ) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

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

pub(super) fn test_thread_context(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let thread =
        match Thread::new(
            3,
            42,
            allocator,
        ) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

    let context =
        thread.context();

    context.rsp()
        == thread.kernel_stack().top()
        && context.rip() == 0
        && context.rflags() == 0x202
        && context.rbx == 0
        && context.rbp == 0
        && context.r12 == 0
        && context.r13 == 0
        && context.r14 == 0
        && context.r15 == 0
}

pub(super) fn test_kernel_context_layout() -> bool {
    use core::mem::{
        size_of,
        offset_of,
    };

    size_of::<crate::thread::KernelContext>() == 72
        && offset_of!(crate::thread::KernelContext, rsp) == 0
        && offset_of!(crate::thread::KernelContext, rip) == 8
        && offset_of!(crate::thread::KernelContext, rflags) == 16
        && offset_of!(crate::thread::KernelContext, rbx) == 24
        && offset_of!(crate::thread::KernelContext, rbp) == 32
        && offset_of!(crate::thread::KernelContext, r12) == 40
        && offset_of!(crate::thread::KernelContext, r13) == 48
        && offset_of!(crate::thread::KernelContext, r14) == 56
        && offset_of!(crate::thread::KernelContext, r15) == 64
}

pub(super) fn test_thread_kernel_stack(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let thread =
        match Thread::new(
            4,
            42,
            allocator,
        ) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

    thread.kernel_stack().size() == KERNEL_STACK_SIZE
        && thread.kernel_stack().base()
            % KERNEL_STACK_ALIGNMENT == 0
        && thread.kernel_stack().top()
            % KERNEL_STACK_ALIGNMENT == 0
        && thread.context().rsp()
            == thread.kernel_stack().top()
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

pub(super) fn run(
    runner: &mut super::framework::TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"thread::creation",
        || test_thread_creation(allocator),
    );

    runner.run(
        b"thread::state",
        || test_thread_state(allocator),
    );

    runner.run(
        b"thread::context",
        || test_thread_context(allocator),
    );

    runner.run(
        b"thread::kernel_context_layout",
        test_kernel_context_layout,
    );

    runner.run(
        b"thread::kernel_stack",
        || test_thread_kernel_stack(allocator),
    );

    runner.run(
        b"thread::kernel_stack_direct",
        test_kernel_stack,
    );

    runner.run(
        b"thread::kernel_stack_validation",
        test_kernel_stack_validation,
    );

    let before =
        allocator.allocated_count();

    let result =
        KernelStack::allocate(allocator)
            .map(|stack| {
                allocator.allocated_count()
                    == before + KERNEL_STACK_PAGES as u64
                    && stack.size() == KERNEL_STACK_SIZE
                    && stack.base()
                        % KERNEL_STACK_ALIGNMENT == 0
                    && stack.top()
                        % KERNEL_STACK_ALIGNMENT == 0
            })
            .unwrap_or(false);

    runner.run(
        b"thread::kernel_stack_allocation",
        || result,
    );
}