use crate::thread::ThreadId;

pub const MAX_RUNNABLE_THREADS: usize = 64;

pub struct RunnableQueue {
    threads: [Option<ThreadId>; MAX_RUNNABLE_THREADS],
    count: usize,
}

impl RunnableQueue {
    pub const fn new() -> Self {
        Self {
            threads: [None; MAX_RUNNABLE_THREADS],
            count: 0,
        }
    }

    pub fn push(
        &mut self,
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        if self.contains(thread_id) {
            return Err(());
        }

        if self.count >= MAX_RUNNABLE_THREADS {
            return Err(());
        }

        self.threads[self.count] = Some(thread_id);
        self.count += 1;

        Ok(())
    }

    pub fn remove(
        &mut self,
        thread_id: ThreadId,
    ) -> Result<(), ()> {
        let index = match self.find(thread_id) {
            Some(index) => index,
            None => return Err(()),
        };

        for i in index..self.count - 1 {
            self.threads[i] =
                self.threads[i + 1];
        }

        self.threads[self.count - 1] = None;
        self.count -= 1;

        Ok(())
    }

    pub fn get(
        &self,
        index: usize,
    ) -> Option<ThreadId> {
        if index >= self.count {
            return None;
        }

        self.threads[index]
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn contains(
        &self,
        thread_id: ThreadId,
    ) -> bool {
        self.find(thread_id).is_some()
    }

    fn find(
        &self,
        thread_id: ThreadId,
    ) -> Option<usize> {
        for index in 0..self.count {
            if self.threads[index] == Some(thread_id) {
                return Some(index);
            }
        }

        None
    }
}