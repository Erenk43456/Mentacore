use crate::memory::physical::PhysicalFrameAllocator;
use crate::process::ProcessId;

use super::{
    KernelContext,
    KernelStack,
};

pub type ThreadId = u64;
pub type ThreadEntry = extern "C" fn() -> !;

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
        entry: ThreadEntry,
    ) -> Result<Self, ()> {
        let kernel_stack =
            KernelStack::allocate(allocator)
                .ok_or(())?;

        /*
         * context_switch restores RSP directly and jumps to RIP.
         *
         * A normal x86_64 function expects RSP % 16 == 8
         * at function entry. Reserve one stack slot so the
         * entry function starts with the expected ABI alignment.
         *
         * The entry function has type `-> !`, so it must never
         * return and therefore does not need a return address.
         */
        let stack_pointer =
            kernel_stack.top() - 8;

        Ok(Self {
            tid,
            process_id,
            state: ThreadState::Ready,
            context: KernelContext::new(
                stack_pointer,
                entry as usize as u64,
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

    pub fn context_mut(&mut self) -> &mut KernelContext {
        &mut self.context
    }

    pub fn kernel_stack(&self) -> &KernelStack {
        &self.kernel_stack
    }

    pub fn set_state(&mut self, state: ThreadState) {
        self.state = state;
    }
}