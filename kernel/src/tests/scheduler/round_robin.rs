use crate::scheduler::Scheduler;

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

pub(super) fn round_robin_remove_current() -> bool {
    let mut scheduler = Scheduler::new();

    scheduler.add(1).is_ok()
        && scheduler.add(2).is_ok()
        && scheduler.add(3).is_ok()
        && scheduler.remove(1).is_ok()
        && scheduler.current() == Some(2)
        && scheduler.next() == Some(3)
        && scheduler.next() == Some(2)
}

pub(super) fn round_robin_empty() -> bool {
    let mut scheduler = Scheduler::new();

    scheduler.next().is_none()
        && scheduler.current().is_none()
}