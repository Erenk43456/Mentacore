use crate::scheduler::RunnableQueue;

pub(super) fn queue_creation() -> bool {
    let queue = RunnableQueue::new();

    queue.is_empty()
        && queue.count() == 0
}

pub(super) fn queue_push() -> bool {
    let mut queue = RunnableQueue::new();

    queue.push(1).is_ok()
        && queue.push(2).is_ok()
        && queue.count() == 2
        && queue.get(0) == Some(1)
        && queue.get(1) == Some(2)
}

pub(super) fn queue_duplicate() -> bool {
    let mut queue = RunnableQueue::new();

    queue.push(1).is_ok()
        && queue.push(1).is_err()
        && queue.count() == 1
}

pub(super) fn queue_remove() -> bool {
    let mut queue = RunnableQueue::new();

    if queue.push(1).is_err() {
        return false;
    }

    if queue.push(2).is_err() {
        return false;
    }

    if queue.push(3).is_err() {
        return false;
    }

    if queue.remove(2).is_err() {
        return false;
    }

    queue.count() == 2
        && queue.get(0) == Some(1)
        && queue.get(1) == Some(3)
}

pub(super) fn queue_missing_remove() -> bool {
    let mut queue = RunnableQueue::new();

    queue.remove(1).is_err()
        && queue.is_empty()
}