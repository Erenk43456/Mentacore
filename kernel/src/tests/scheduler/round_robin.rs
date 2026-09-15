use crate::memory::physical::PhysicalFrameAllocator;
use crate::scheduler::Scheduler;
use crate::thread::ThreadManager;

extern "C" fn scheduler_test_entry() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub(super) fn round_robin_selection() -> bool {
    let mut scheduler = Scheduler::new();

    if scheduler.add(1).is_err() {
        return false;
    }

    if scheduler.add(2).is_err() {
        return false;
    }

    if scheduler.add(3).is_err() {
        return false;
    }

    scheduler.current() == Some(1)
        && scheduler.next() == Some(2)
        && scheduler.next() == Some(3)
        && scheduler.next() == Some(1)
        && scheduler.next() == Some(2)
}

pub(super) fn round_robin_remove_current(
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

    scheduler.start(&mut manager) == Some(1)
        && scheduler.remove(&mut manager, 1).is_ok()
        && scheduler.current() == Some(2)
        && scheduler.next() == Some(3)
        && scheduler.next() == Some(2)
}

pub(super) fn round_robin_empty() -> bool {
    let mut scheduler = Scheduler::new();

    scheduler.next().is_none()
        && scheduler.current().is_none()
}