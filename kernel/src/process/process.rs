use crate::memory::LoadedElf;
use crate::memory::paging::AddressSpace;
use crate::memory::physical::PhysicalFrameAllocator;

pub type ProcessId = u64;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

pub struct Process {
    pid: ProcessId,
    state: ProcessState,
    address_space: AddressSpace,
    entry: u64,
    user_stack_top: u64,
}

impl Process {
    pub unsafe fn new(
        pid: ProcessId,
        allocator: &mut PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let address_space =
            unsafe { AddressSpace::new(allocator)? };

        Ok(Self {
            pid,
            state: ProcessState::Ready,
            address_space,
            entry: 0,
            user_stack_top: 0,
        })
    }

    pub unsafe fn from_loaded_elf(
        pid: ProcessId,
        loaded_elf: LoadedElf,
        user_stack_top: u64,
    ) -> Self {
        Self {
            pid,
            state: ProcessState::Ready,
            entry: loaded_elf.entry(),
            address_space: loaded_elf.into_address_space(),
            user_stack_top,
        }
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
    }

    pub fn address_space_mut(&mut self) -> &mut AddressSpace {
        &mut self.address_space
    }

    pub fn entry(&self) -> u64 {
        self.entry
    }

    pub fn user_stack_top(&self) -> u64 {
        self.user_stack_top
    }

    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
}