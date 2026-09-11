use crate::memory::physical::PhysicalFrameAllocator;
use crate::process::ProcessId;
use crate::thread::{
    ThreadManager,
    ThreadState,
    ThreadId,
};

use super::Scheduler;

pub const IDLE_THREAD_ID: ThreadId = 0;
pub const KERNEL_PROCESS_ID: ProcessId = 0;

extern "C" fn idle_thread() -> ! {
    loop {
        crate::cpu::halt();
    }
}

pub struct SchedulerRuntime {
    scheduler: Scheduler,
    manager: ThreadManager,
}

impl SchedulerRuntime {
    pub fn new(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let mut manager = ThreadManager::new();
        let mut scheduler = Scheduler::new();

        manager.create(
            IDLE_THREAD_ID,
            KERNEL_PROCESS_ID,
            allocator,
            idle_thread,
        )?;

        scheduler.add_thread(
            &manager,
            IDLE_THREAD_ID,
        )?;

        scheduler.start(&mut manager);

        Ok(Self {
            scheduler,
            manager,
        })
    }

    pub fn scheduler(&self) -> &Scheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    pub fn thread_manager(&self) -> &ThreadManager {
        &self.manager
    }

    pub fn thread_manager_mut(
        &mut self,
    ) -> &mut ThreadManager {
        &mut self.manager
    }

    pub fn current(&self) -> Option<ThreadId> {
        self.scheduler.current()
    }

    pub fn current_state(&self) -> Option<ThreadState> {
        self.current()
            .and_then(|thread_id|
                self.manager
                    .get(thread_id)
                    .map(|thread| thread.state())
            )
    }
}