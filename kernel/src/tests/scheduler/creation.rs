use crate::scheduler::Scheduler;

pub(super) fn scheduler_creation() -> bool {
    let scheduler = Scheduler::new();

    scheduler.is_empty()
        && scheduler.count() == 0
        && scheduler.current().is_none()
}