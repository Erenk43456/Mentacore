use crate::thread::{
    ThreadId,
    ThreadManager,
};

use super::queue::RunnableQueue;

pub struct Scheduler {
    queue: RunnableQueue,
    current: Option<usize>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            queue: RunnableQueue::new(),
            current: None,
        }
    }

    pub fn add(
        &mut self,
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        self.queue.push(thread_id)?;

        if self.current.is_none() {
            self.current = Some(0);
        }

        Ok(())
    }

    pub fn add_thread(
        &mut self,
        manager: &ThreadManager,
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        if !manager.contains(thread_id) {
            return Err(());
        }

        self.add(thread_id)
    }

    pub fn start(
        &mut self,
        manager: &mut ThreadManager,
    ) -> Option<ThreadId> {
        let current = self.current()?;

        if let Some(thread) = manager.get_mut(current) {
            thread.set_state(
                crate::thread::ThreadState::Running
            );
        }

        Some(current)
    }

    pub fn schedule_next(
        &mut self,
        manager: &mut ThreadManager,
    ) -> Option<ThreadId> {
        let previous = self.current();

        let next = self.next()?;

        if let Some(previous) = previous {
            if previous != next {
                if let Some(thread) =
                    manager.get_mut(previous)
                {
                    thread.set_state(
                        crate::thread::ThreadState::Ready
                    );
                }
            }
        }

        if let Some(thread) = manager.get_mut(next) {
            thread.set_state(
                crate::thread::ThreadState::Running
            );
        }

        Some(next)
    }

    pub fn remove(
        &mut self,
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        let removed_index =
            self.find_index(thread_id)
                .ok_or(())?;

        self.queue.remove(thread_id)?;

        match self.current {
            None => {}

            Some(current)
                if self.queue.is_empty() =>
            {
                self.current = None;
            }

            Some(current)
                if removed_index < current =>
            {
                self.current = Some(current - 1);
            }

            Some(current)
                if removed_index == current
                    && current >= self.queue.count() =>
            {
                self.current = Some(0);
            }

            _ => {}
        }

        Ok(())
    }

    pub fn current(&self) -> Option<ThreadId> {
        self.current
            .and_then(|index| self.queue.get(index))
    }

    pub fn next(&mut self) -> Option<ThreadId> {
        let count = self.queue.count();

        if count == 0 {
            self.current = None;
            return None;
        }

        let next_index = match self.current {
            Some(current) =>
                (current + 1) % count,

            None => 0,
        };

        self.current = Some(next_index);

        self.queue.get(next_index)
    }

    pub fn count(&self) -> usize {
        self.queue.count()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn find_index(
        &self,
        thread_id: ThreadId,
    ) -> Option<usize> {
        for index in 0..self.queue.count() {
            if self.queue.get(index) == Some(thread_id) {
                return Some(index);
            }
        }

        None
    }
}