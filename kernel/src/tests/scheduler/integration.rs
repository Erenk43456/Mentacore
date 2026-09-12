use core::sync::atomic::{
    AtomicU64,
    Ordering,
};

use crate::memory::physical::PhysicalFrameAllocator;
use crate::scheduler::{
    Scheduler,
    SchedulerRuntime,
    IDLE_THREAD_ID,
};
use crate::thread::{
    ThreadManager,
    ThreadState,
};

static TIMER_PREEMPTION_WORKER_RUNS: AtomicU64 =
    AtomicU64::new(0);

extern "C" fn timer_preemption_worker() -> ! {
    loop {
        TIMER_PREEMPTION_WORKER_RUNS.fetch_add(
            1,
            Ordering::Relaxed,
        );

        crate::cpu::halt();
    }
}

pub(super) fn prepare_timer_preemption(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    TIMER_PREEMPTION_WORKER_RUNS.store(
        0,
        Ordering::Relaxed,
    );

    SchedulerRuntime::prepare_thread(
        1,
        crate::scheduler::KERNEL_PROCESS_ID,
        allocator,
        timer_preemption_worker,
    )
    .is_ok()
}

extern "C" fn scheduler_test_entry() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub(super) fn scheduler_accepts_managed_thread(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager = ThreadManager::new();
    let mut scheduler = Scheduler::new();

    if manager.create(
        1,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if scheduler
        .add_thread(&manager, 1)
        .is_err()
    {
        return false;
    }

    scheduler.current() == Some(1)
        && scheduler.count() == 1
}

pub(super) fn scheduler_rejects_unknown_thread(
    _allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let manager = ThreadManager::new();
    let mut scheduler = Scheduler::new();

    scheduler
        .add_thread(&manager, 1)
        .is_err()
        && scheduler.is_empty()
}

pub(super) fn scheduler_selects_managed_threads(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager = ThreadManager::new();
    let mut scheduler = Scheduler::new();

    if manager.create(
        1,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        2,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        3,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if scheduler.add_thread(&manager, 1).is_err() {
        return false;
    }

    if scheduler.add_thread(&manager, 2).is_err() {
        return false;
    }

    if scheduler.add_thread(&manager, 3).is_err() {
        return false;
    }

    scheduler.current() == Some(1)
        && scheduler.next() == Some(2)
        && scheduler.next() == Some(3)
        && scheduler.next() == Some(1)
}

pub(super) fn scheduler_manages_thread_states(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager =
        ThreadManager::new();

    let mut scheduler =
        Scheduler::new();

    if manager.create(
        1,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        2,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if scheduler
        .add_thread(&manager, 1)
        .is_err()
    {
        return false;
    }

    if scheduler
        .add_thread(&manager, 2)
        .is_err()
    {
        return false;
    }

    if scheduler.start(&mut manager) != Some(1) {
        return false;
    }

    if manager
        .get(1)
        .map(|thread| {
            thread.state()
                == crate::thread::ThreadState::Running
        })
        != Some(true)
    {
        return false;
    }

    if manager
        .get(2)
        .map(|thread| {
            thread.state()
                == crate::thread::ThreadState::Ready
        })
        != Some(true)
    {
        return false;
    }

    if scheduler.schedule_next(&mut manager)
        != Some(2)
    {
        return false;
    }

    manager
        .get(1)
        .map(|thread| {
            thread.state()
                == crate::thread::ThreadState::Ready
        })
        == Some(true)
        &&
    manager
        .get(2)
        .map(|thread| {
            thread.state()
                == crate::thread::ThreadState::Running
        })
        == Some(true)
}

static mut SCHEDULER_SWITCH_RETURN_CONTEXT:
    *const crate::thread::KernelContext =
    core::ptr::null();

static mut SCHEDULER_SWITCH_TARGET:
    crate::thread::KernelContext =
    crate::thread::KernelContext::new(0, 0);

static mut SCHEDULER_SWITCH_REACHED: bool = false;

#[unsafe(no_mangle)]
extern "C" fn scheduler_context_switch_entry() -> ! {
    unsafe {
        SCHEDULER_SWITCH_REACHED = true;

        crate::thread::context_switch(
            &raw mut SCHEDULER_SWITCH_TARGET,
            SCHEDULER_SWITCH_RETURN_CONTEXT,
        );
    }

    loop {
        core::hint::spin_loop();
    }
}

pub(super) fn scheduler_switches_to_selected_thread(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager = ThreadManager::new();
    let mut scheduler = Scheduler::new();

    if manager.create(
        1,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        2,
        42,
        allocator,
        scheduler_context_switch_entry,
    ).is_err() {
        return false;
    }

    if scheduler
        .add_thread(&manager, 1)
        .is_err()
    {
        return false;
    }

    if scheduler
        .add_thread(&manager, 2)
        .is_err()
    {
        return false;
    }

    if scheduler.start(&mut manager) != Some(1) {
        return false;
    }

    let return_context =
        match manager.get(1) {
            Some(thread) =>
                core::ptr::addr_of!(*thread.context()),
            None => return false,
        };

    unsafe {
        SCHEDULER_SWITCH_RETURN_CONTEXT =
            return_context;
        SCHEDULER_SWITCH_REACHED = false;

        if scheduler.switch_to_next(&mut manager) != Some(2) {
            return false;
        }

        return SCHEDULER_SWITCH_REACHED;
    }
}

pub(super) fn scheduler_runtime_starts_idle_thread(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let runtime =
        match SchedulerRuntime::new(allocator) {
            Ok(runtime) => runtime,
            Err(()) => return false,
        };

    if runtime.current()
        != Some(IDLE_THREAD_ID)
    {
        return false;
    }

    if runtime.scheduler().count() != 1 {
        return false;
    }

    if runtime
        .thread_manager()
        .get(IDLE_THREAD_ID)
        .is_none()
    {
        return false;
    }

    runtime.current_state()
        == Some(
            crate::thread::ThreadState::Running
        )
}

pub(super) fn scheduler_preempts_thread(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager =
        ThreadManager::new();

    let mut scheduler =
        Scheduler::new();

    if manager.create(
        1,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if manager.create(
        2,
        42,
        allocator,
        scheduler_test_entry,
    ).is_err() {
        return false;
    }

    if scheduler
        .add_thread(&manager, 1)
        .is_err()
    {
        return false;
    }

    if scheduler
        .add_thread(&manager, 2)
        .is_err()
    {
        return false;
    }

    if scheduler.start(&mut manager)
        != Some(1)
    {
        return false;
    }

    /*
     * Simulate the RSP supplied by the LAPIC
     * interrupt entry path.
     */
    let current_rsp =
        0x001d_0000_u64;

    let next_rsp =
        match scheduler.preempt(
            &mut manager,
            current_rsp,
        ) {
            Some(rsp) => rsp,
            None => return false,
        };

    /*
     * Thread 1 must now contain the interrupt
     * frame belonging to the interrupted context.
     */
    if manager
        .get(1)
        .map(|thread| {
            thread.interrupt_rsp()
                == current_rsp
        })
        != Some(true)
    {
        return false;
    }

    /*
     * Scheduler must have selected thread 2.
     */
    if scheduler.current() != Some(2) {
        return false;
    }

    /*
     * Thread 1 becomes Ready and thread 2
     * becomes Running.
     */
    if manager
        .get(1)
        .map(|thread| {
            thread.state()
                == ThreadState::Ready
        })
        != Some(true)
    {
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

    /*
     * The returned RSP must be the pre-built
     * InterruptContext of thread 2.
     */
    next_rsp != 0
        && next_rsp
            == manager
                .get(2)
                .map(|thread|
                    thread.interrupt_rsp()
                )
                .unwrap_or(0)
}

pub(super) fn scheduler_timer_preemption() -> bool {
    if SchedulerRuntime::activate_thread(1).is_err() {
        return false;
    }

    for _ in 0..5_000_000 {
        if TIMER_PREEMPTION_WORKER_RUNS.load(
            Ordering::Relaxed,
        ) >= 2
        {
            return true;
        }

        core::hint::spin_loop();
    }

    false
}