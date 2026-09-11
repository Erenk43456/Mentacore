use crate::thread::{
    Thread,
    ThreadState,
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
}