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
    interrupt_rsp: u64,
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

        let mut thread = Self {
            tid,
            process_id,
            state: ThreadState::Ready,
            context: KernelContext::new(
                stack_pointer,
                entry as usize as u64,
            ),
            kernel_stack,
            interrupt_rsp: 0,
        };

        thread.prepare_interrupt_context()?;

        Ok(thread)
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

    pub fn interrupt_rsp(&self) -> u64 {
        self.interrupt_rsp
    }

    pub fn set_interrupt_rsp(
        &mut self,
        rsp: u64,
    ) {
        self.interrupt_rsp = rsp;
    }

    pub fn prepare_interrupt_context(
        &mut self,
    ) -> Result<(), ()> {
        let frame_size =
            core::mem::size_of::<super::InterruptContext>()
                as u64;

        /*
        * Keep the post-iret stack aligned exactly like a
        * normal kernel thread entry.
        *
        * iretq consumes 144 bytes, leaving:
        *
        *     RSP = stack_top - 8
        *
        * which matches the KernelContext startup ABI.
        */
        let frame_address =
            self.kernel_stack
                .top()
                .checked_sub(8)
                .ok_or(())?
                .checked_sub(frame_size)
                .ok_or(())?;

        let frame =
            super::InterruptContext::new(
                self.context.rip(),
                0x08,
                self.context.rflags(),
            );

        unsafe {
            core::ptr::write(
                frame_address
                    as *mut super::InterruptContext,
                frame,
            );
        }

        self.interrupt_rsp =
            frame_address;

        Ok(())
    }
}