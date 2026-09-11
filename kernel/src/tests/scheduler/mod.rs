mod creation;
mod queue;
mod round_robin;

use super::framework::TestRunner;

pub(super) fn run(
    runner: &mut TestRunner,
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
}