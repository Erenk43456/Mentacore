use crate::process::ProcessId;

use super::KernelContext;

pub type ThreadId = u64;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

pub struct Thread {
    tid: ThreadId,
    process_id: ProcessId,
    state: ThreadState,
    context: KernelContext,
}

impl Thread {
    pub fn new(
        tid: ThreadId,
        process_id: ProcessId,
    ) -> Self {
        Self {
            tid,
            process_id,
            state: ThreadState::Ready,
            context: KernelContext::new(0, 0),
        }
    }

    pub fn tid(&self) -> ThreadId {
        self.tid
    }

    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    pub fn state(&self) -> ThreadState {
        self.state
    }

    pub fn context(&self) -> &KernelContext {
        &self.context
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }
}