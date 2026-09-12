use crate::scheduler::SchedulerRuntime;

pub fn get_tid() -> u64 {
    SchedulerRuntime::current_thread_id()
        .map(|thread_id| thread_id as u64)
        .unwrap_or(u64::MAX)
}