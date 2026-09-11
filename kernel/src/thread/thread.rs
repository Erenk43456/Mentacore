use crate::memory::physical::PhysicalFrameAllocator;
use crate::process::ProcessId;

use super::{
    KernelContext,
    KernelStack,
};

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
    kernel_stack: KernelStack,
}

impl Thread {
    pub fn new(
        tid: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let kernel_stack =
            KernelStack::allocate(allocator)
                .ok_or(())?;

        Ok(Self {
            tid,
            process_id,
            state: ThreadState::Ready,
            context: KernelContext::new(
                kernel_stack.top(),
                0,
            ),
            kernel_stack,
        })
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

    pub fn kernel_stack(&self) -> &KernelStack {
        &self.kernel_stack
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }
}