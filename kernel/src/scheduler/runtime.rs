use crate::memory::physical::PhysicalFrameAllocator;
use crate::process::ProcessId;
use crate::thread::{
    ThreadManager,
    ThreadState,
    ThreadId,
};
use crate::sync::Spinlock;

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

pub static SCHEDULER_RUNTIME:
    Spinlock<Option<SchedulerRuntime>> =
    Spinlock::new(None);

impl SchedulerRuntime {
    pub fn initialize(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<(), ()> {
        let runtime =
            SchedulerRuntime::new(allocator)?;

        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        if guard.is_some() {
            return Err(());
        }

        *guard = Some(runtime);

        Ok(())
    }

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

    pub fn current_thread_id() -> Option<ThreadId> {
        let guard = SCHEDULER_RUNTIME.lock_irqsave();
        guard.as_ref().and_then(|runtime| runtime.current())
    }

    pub fn current_state(&self) -> Option<ThreadState> {
        self.current()
            .and_then(|thread_id|
                self.manager
                    .get(thread_id)
                    .map(|thread| thread.state())
            )
    }

    pub fn create_thread(
        thread_id: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
        entry: crate::thread::ThreadEntry,
    ) -> Result<(), ()> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut().ok_or(())?;

        runtime.manager.create(
            thread_id,
            process_id,
            allocator,
            entry,
        )?;

        runtime.scheduler.add_thread(
            &runtime.manager,
            thread_id,
        )?;

        Ok(())
    }

    pub fn prepare_thread(
        thread_id: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
        entry: crate::thread::ThreadEntry,
    ) -> Result<(), ()> {
        let mut guard = SCHEDULER_RUNTIME.lock_irqsave();
        let runtime = guard.as_mut().ok_or(())?;

        runtime
            .manager
            .create(
                thread_id,
                process_id,
                allocator,
                entry,
            )?;

        Ok(())
    }

    pub fn activate_thread(
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        let mut guard = SCHEDULER_RUNTIME.lock_irqsave();
        let runtime = guard.as_mut().ok_or(())?;

        runtime
            .scheduler
            .add_thread(
                &runtime.manager,
                thread_id,
            )?;

        Ok(())
    }

    pub fn preempt(
        current_rsp: u64,
    ) -> Option<u64> {
        let mut guard =
            SCHEDULER_RUNTIME.lock_irqsave();

        let runtime =
            guard.as_mut()?;

        let next_rsp =
            runtime.scheduler.preempt(
                &mut runtime.manager,
                current_rsp,
            )?;

        drop(guard);

        Some(next_rsp)
    }
}