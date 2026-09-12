use crate::memory::LoadedElf;

use super::{
    Process,
    ProcessId,
};

pub const MAX_PROCESSES: usize = 16;

pub struct ProcessManager {
    processes: [Option<Process>; MAX_PROCESSES],
    count: usize,
}

impl ProcessManager {
    pub const fn new() -> Self {
        Self {
            processes: [const { None }; MAX_PROCESSES],
            count: 0,
        }
    }

    pub fn create_userspace(
        &mut self,
        process_id: ProcessId,
        loaded_elf: LoadedElf,
        user_stack_top: u64,
    ) -> Result<(), ()> {
        if self.count >= MAX_PROCESSES {
            return Err(());
        }

        if self.contains(process_id) {
            return Err(());
        }

        let process = unsafe {
            Process::from_loaded_elf(
                process_id,
                loaded_elf,
                user_stack_top,
            )
        };

        for slot in self.processes.iter_mut() {
            if slot.is_none() {
                *slot = Some(process);
                self.count += 1;
                return Ok(());
            }
        }

        Err(())
    }

    pub fn get(
        &self,
        process_id: ProcessId,
    ) -> Option<&Process> {
        self.processes
            .iter()
            .flatten()
            .find(|process| process.pid() == process_id)
    }

    pub fn get_mut(
        &mut self,
        process_id: ProcessId,
    ) -> Option<&mut Process> {
        self.processes
            .iter_mut()
            .flatten()
            .find(|process| process.pid() == process_id)
    }

    pub fn contains(
        &self,
        process_id: ProcessId,
    ) -> bool {
        self.get(process_id).is_some()
    }

    pub fn count(&self) -> usize {
        self.count
    }
}