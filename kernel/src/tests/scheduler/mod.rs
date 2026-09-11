mod creation;
mod integration;
mod queue;
mod round_robin;

use crate::tests::scheduler::integration::scheduler_manages_thread_states;
use crate::memory::physical::PhysicalFrameAllocator;

use super::framework::TestRunner;

pub(super) fn run(
    runner: &mut TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"scheduler::creation",
        creation::scheduler_creation,
    );

    runner.run(
        b"scheduler::queue_creation",
        queue::queue_creation,
    );

    runner.run(
        b"scheduler::queue_push",
        queue::queue_push,
    );

    runner.run(
        b"scheduler::queue_duplicate",
        queue::queue_duplicate,
    );

    runner.run(
        b"scheduler::queue_remove",
        queue::queue_remove,
    );

    runner.run(
        b"scheduler::queue_missing_remove",
        queue::queue_missing_remove,
    );

    runner.run(
        b"scheduler::round_robin_selection",
        round_robin::round_robin_selection,
    );

    runner.run(
        b"scheduler::round_robin_remove_current",
        round_robin::round_robin_remove_current,
    );

    runner.run(
        b"scheduler::round_robin_empty",
        round_robin::round_robin_empty,
    );

    runner.run(
        b"scheduler::managed_thread",
        || integration::scheduler_accepts_managed_thread(allocator),
    );

    runner.run(
        b"scheduler::unknown_thread",
        || integration::scheduler_rejects_unknown_thread(allocator),
    );

    runner.run(
        b"scheduler::managed_thread_selection",
        || integration::scheduler_selects_managed_threads(allocator),
    );

    runner.run(
        b"scheduler::thread_states",
        || {
            scheduler_manages_thread_states(allocator)
        },
    );
}