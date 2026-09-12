use super::{
    Thread,
    ThreadEntry,
    ThreadId,
};

use crate::memory::physical::PhysicalFrameAllocator;
use crate::process::ProcessId;

pub const MAX_THREADS: usize = 64;

pub struct ThreadManager {
    threads: [Option<Thread>; MAX_THREADS],
    count: usize,
}

impl ThreadManager {
    pub const fn new() -> Self {
        Self {
            threads: [const { None }; MAX_THREADS],
            count: 0,
        }
    }

    pub fn create(
        &mut self,
        thread_id: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
        entry: ThreadEntry,
    ) -> Result<(), ()> {
        if self.count >= MAX_THREADS {
            return Err(());
        }

        if self.contains(thread_id) {
            return Err(());
        }

        let thread =
            Thread::new(
                thread_id,
                process_id,
                allocator,
                entry,
            )?;

        for slot in self.threads.iter_mut() {
            if slot.is_none() {
                *slot = Some(thread);
                self.count += 1;
                return Ok(());
            }
        }

        Err(())
    }

    pub fn remove(
        &mut self,
        thread_id: ThreadId,
    ) -> Result<Thread, ()> {
        for slot in self.threads.iter_mut() {
            if slot
                .as_ref()
                .map(|thread| thread.tid() == thread_id)
                .unwrap_or(false)
            {
                let thread =
                    slot.take().ok_or(())?;

                self.count -= 1;

                return Ok(thread);
            }
        }

        Err(())
    }

    pub fn get(
        &self,
        thread_id: ThreadId,
    ) -> Option<&Thread> {
        for slot in self.threads.iter() {
            if let Some(thread) = slot {
                if thread.tid() == thread_id {
                    return Some(thread);
                }
            }
        }

        None
    }

    pub fn get_mut(
        &mut self,
        thread_id: ThreadId,
    ) -> Option<&mut Thread> {
        for slot in self.threads.iter_mut() {
            if let Some(thread) = slot {
                if thread.tid() == thread_id {
                    return Some(thread);
                }
            }
        }

        None
    }

    pub fn contains(
        &self,
        thread_id: ThreadId,
    ) -> bool {
        self.get(thread_id).is_some()
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn create_user(
        &mut self,
        thread_id: ThreadId,
        process_id: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
        entry: u64,
        user_stack_top: u64,
    ) -> Result<(), ()> {
        if self.count >= MAX_THREADS {
            return Err(());
        }

        if self.contains(thread_id) {
            return Err(());
        }

        let thread =
            Thread::new_user(
                thread_id,
                process_id,
                allocator,
                entry,
                user_stack_top,
            )?;

        for slot in self.threads.iter_mut() {
            if slot.is_none() {
                *slot = Some(thread);
                self.count += 1;
                return Ok(());
            }
        }

        Err(())
    }
}