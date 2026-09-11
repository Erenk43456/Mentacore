use crate::memory::physical::PhysicalFrameAllocator;
use crate::scheduler::Scheduler;
use crate::thread::ThreadManager;

pub(super) fn scheduler_accepts_managed_thread(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut manager = ThreadManager::new();
    let mut scheduler = Scheduler::new();

    if manager.create(1, 42, allocator).is_err() {
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

    if manager.create(1, 42, allocator).is_err() {
        return false;
    }

    if manager.create(2, 42, allocator).is_err() {
        return false;
    }

    if manager.create(3, 42, allocator).is_err() {
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

    if manager.create(1, 42, allocator).is_err() {
        return false;
    }

    if manager.create(2, 42, allocator).is_err() {
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