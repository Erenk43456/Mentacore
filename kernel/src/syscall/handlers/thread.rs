use crate::scheduler::SchedulerRuntime;
use crate::syscall::result::SyscallResult;

pub fn get_tid() -> SyscallResult {
    SchedulerRuntime::current_thread_id()
        .map(|thread_id| thread_id as u64)
        .unwrap_or(crate::syscall::result::SYSCALL_ERROR)
}