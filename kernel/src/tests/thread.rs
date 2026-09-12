use crate::memory::physical::PhysicalFrameAllocator;
use crate::thread::{
    KernelStack,
    KERNEL_STACK_ALIGNMENT,
    KERNEL_STACK_PAGES,
    KERNEL_STACK_SIZE,
    Thread,
    ThreadManager,
    ThreadState,
    InterruptContext,
};

extern "C" fn test_thread_entry() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

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
            test_thread_entry,
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
            test_thread_entry,
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
            test_thread_entry,
        ) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

    let context =
        thread.context();

    context.rsp()
        == thread.kernel_stack().top() - 8
        && context.rip()
            == test_thread_entry as usize as u64
        && context.rflags() == 0x202
        && context.rbx == 0
        && context.rbp == 0
        && context.r12 == 0
        && context.r13 == 0
        && context.r14 == 0
        && context.r15 == 0
}

fn test_interrupt_context(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    crate::debug::write(b"[IC] before Thread::new\r\n");

    let thread =
        match Thread::new(
            6,
            42,
            allocator,
            test_thread_entry,
        ) {
            Ok(thread) => thread,
            Err(()) => {
                crate::debug::write(
                    b"[IC] Thread::new FAILED\r\n",
                );
                return false;
            }
        };

    crate::debug::write(
        b"[IC] after Thread::new\r\n",
    );

    let interrupt_rsp =
        thread.interrupt_rsp();

    crate::debug::write(
        b"[IC] interrupt_rsp obtained\r\n",
    );

    if interrupt_rsp == 0 {
        return false;
    }

    let frame = unsafe {
        &*(interrupt_rsp as *const InterruptContext)
    };

    frame.rip == test_thread_entry as usize as u64
        && frame.cs == 0x08
        && frame.rflags == 0x202
        && frame.rsp == thread.kernel_stack().top() - 8
        && frame.ss == 0x10
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

static mut CONTEXT_SWITCH_CURRENT: crate::thread::KernelContext =
    crate::thread::KernelContext::new(0, 0);

static mut CONTEXT_SWITCH_NEXT: crate::thread::KernelContext =
    crate::thread::KernelContext::new(0, 0);

static mut CONTEXT_SWITCH_STACK: [u8; 4096] = [0; 4096];

static mut CONTEXT_SWITCH_RETURNED: bool = false;

unsafe extern "C" {
    fn context_switch_test_trampoline();
}

#[unsafe(no_mangle)]
extern "C" fn context_switch_test_target() -> ! {
    unsafe {
        CONTEXT_SWITCH_RETURNED = true;

        crate::thread::context_switch(
            &raw mut CONTEXT_SWITCH_NEXT,
            &raw const CONTEXT_SWITCH_CURRENT,
        );
    }

    loop {
        core::hint::spin_loop();
    }
}

static mut THREAD_STARTUP_CURRENT:
    crate::thread::KernelContext =
    crate::thread::KernelContext::new(0, 0);

static mut THREAD_STARTUP_CONTEXT:
    crate::thread::KernelContext =
    crate::thread::KernelContext::new(0, 0);

static mut THREAD_STARTUP_REACHED: bool = false;

#[unsafe(no_mangle)]
extern "C" fn thread_startup_entry() -> ! {
    unsafe {
        THREAD_STARTUP_REACHED = true;

        crate::thread::context_switch(
            &raw mut THREAD_STARTUP_CONTEXT,
            &raw const THREAD_STARTUP_CURRENT,
        );
    }

    loop {
        core::hint::spin_loop();
    }
}

pub(super) fn test_context_switch() -> bool {
    let stack_base =
        core::ptr::addr_of!(CONTEXT_SWITCH_STACK)
            as *const u8;

    let stack_top =
        stack_base.wrapping_add(4096) as u64;

    /*
     * The trampoline executes CALL, so RSP must be
     * 16-byte aligned before the CALL.
     */
    let stack_pointer =
        stack_top & !0xF;

    let trampoline =
        context_switch_test_trampoline
            as *const ()
            as usize
            as u64;

    unsafe {
        CONTEXT_SWITCH_RETURNED = false;

        CONTEXT_SWITCH_CURRENT =
            crate::thread::KernelContext::new(
                0,
                0,
            );

        CONTEXT_SWITCH_NEXT =
            crate::thread::KernelContext::new(
                stack_pointer,
                trampoline,
            );

        let current_ptr =
            core::ptr::addr_of!(CONTEXT_SWITCH_CURRENT);

        let next_ptr =
            core::ptr::addr_of!(CONTEXT_SWITCH_NEXT);

        crate::debug::write(b"CURRENT RSP: ");
        crate::debug::write_hex(
            unsafe { core::ptr::read(core::ptr::addr_of!((*current_ptr).rsp)) },
        );

        crate::debug::write(b"CURRENT RIP: ");
        crate::debug::write_hex(
            unsafe { core::ptr::read(core::ptr::addr_of!((*current_ptr).rip)) },
        );

        crate::debug::write(b"NEXT RSP: ");
        crate::debug::write_hex(
            unsafe { core::ptr::read(core::ptr::addr_of!((*next_ptr).rsp)) },
        );

        crate::debug::write(b"NEXT RIP: ");
        crate::debug::write_hex(
            unsafe { core::ptr::read(core::ptr::addr_of!((*next_ptr).rip)) },
        );

        crate::debug::write(b"TRAMPOLINE: ");
        crate::debug::write_hex(trampoline);

        crate::thread::context_switch(
            &raw mut CONTEXT_SWITCH_CURRENT,
            &raw const CONTEXT_SWITCH_NEXT,
        );

        CONTEXT_SWITCH_RETURNED
    }
}

pub(super) fn test_thread_context_startup(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let thread =
        match Thread::new(
            5,
            42,
            allocator,
            thread_startup_entry,
        ) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

    unsafe {
        THREAD_STARTUP_REACHED = false;

        THREAD_STARTUP_CURRENT =
            crate::thread::KernelContext::new(
                0,
                0,
            );

        THREAD_STARTUP_CONTEXT =
            *thread.context();

        crate::thread::context_switch(
            &raw mut THREAD_STARTUP_CURRENT,
            &raw const THREAD_STARTUP_CONTEXT,
        );

        THREAD_STARTUP_REACHED
    }
}

pub(super) fn test_thread_kernel_stack(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let thread =
        match Thread::new(
            4,
            42,
            allocator,
            test_thread_entry,
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
            == thread.kernel_stack().top() - 8
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
        b"thread::interrupt_context",
        || test_interrupt_context(allocator),
    );

    runner.run(
        b"thread::kernel_context_layout",
        test_kernel_context_layout,
    );

    runner.run(
        b"thread::context_switch",
        test_context_switch,
    );

    runner.run(
        b"thread::context_startup",
        || test_thread_context_startup(allocator),
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

    runner.run(
        b"thread::manager_creation",
        test_thread_manager_creation,
    );

    runner.run(
        b"thread::manager_operations",
        || test_thread_manager_operations(allocator),
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

pub(super) fn test_thread_manager_creation() -> bool {
    let manager = crate::thread::ThreadManager::new();

    manager.is_empty()
        && manager.count() == 0
}

pub(super) fn test_thread_manager_operations(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager =
        crate::thread::ThreadManager::new();

    if manager.create(
        1,
        42,
        allocator,
        test_thread_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        2,
        42,
        allocator,
        test_thread_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        1,
        42,
        allocator,
        test_thread_entry,
    ).is_ok() {
        return false;
    }

    if manager.count() != 2 {
        return false;
    }

    if !manager.contains(1)
        || !manager.contains(2)
        || manager.contains(3)
    {
        return false;
    }

    if manager
        .get(1)
        .map(|thread| {
            thread.tid() == 1
                && thread.process_id() == 42
        })
        != Some(true)
    {
        return false;
    }

    if let Some(thread) = manager.get_mut(2) {
        thread.set_state(
            ThreadState::Running
        );
    } else {
        return false;
    }

    if manager
        .get(2)
        .map(|thread| {
            thread.state()
                == ThreadState::Running
        })
        != Some(true)
    {
        return false;
    }

    let removed =
        match manager.remove(1) {
            Ok(thread) => thread,
            Err(()) => return false,
        };

    removed.tid() == 1
        && manager.count() == 1
        && !manager.contains(1)
        && manager.contains(2)
        && manager.remove(1).is_err()
}