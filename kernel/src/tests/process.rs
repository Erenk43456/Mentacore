use crate::memory::physical::PhysicalFrameAllocator;
use crate::process::{
    Process,
    ProcessState,
};

pub(super) fn test_process_creation(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let process =
        unsafe {
            match Process::new(1, allocator) {
                Ok(process) => process,
                Err(()) => return false,
            }
        };

    process.pid() == 1
        && process.state() == ProcessState::Ready
        && !process.address_space().pml4().is_null()
}

pub(super) fn test_process_state(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let mut process =
        unsafe {
            match Process::new(2, allocator) {
                Ok(process) => process,
                Err(()) => return false,
            }
        };

    if process.state() != ProcessState::Ready {
        return false;
    }

    process.set_state(ProcessState::Running);

    if process.state() != ProcessState::Running {
        return false;
    }

    process.set_state(ProcessState::Blocked);

    if process.state() != ProcessState::Blocked {
        return false;
    }

    process.set_state(ProcessState::Terminated);

    process.state() == ProcessState::Terminated
}

pub(super) fn run(
    runner: &mut super::framework::TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"process::creation",
        || test_process_creation(allocator),
    );

    runner.run(
        b"process::state",
        || test_process_state(allocator),
    );
}