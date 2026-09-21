use crate::memory::LoadedElf;
use crate::memory::paging::AddressSpace;

#[cfg(feature = "kernel-tests")]
use crate::memory::physical::PhysicalFrameAllocator;

pub type ProcessId = u64;

#[cfg(feature = "kernel-tests")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

pub struct Process {
    pid: ProcessId,

    #[cfg(feature = "kernel-tests")]
    state: ProcessState,

    address_space: AddressSpace,
    entry: u64,
    user_stack_top: u64,
}

impl Process {
    #[cfg(feature = "kernel-tests")]
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

            #[cfg(feature = "kernel-tests")]
            state: ProcessState::Ready,

            entry: loaded_elf.entry(),
            address_space: loaded_elf.into_address_space(),
            user_stack_top,
        }
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[cfg(feature = "kernel-tests")]
    pub fn state(&self) -> ProcessState {
        self.state
    }

    pub fn address_space(&self) -> &AddressSpace {
        &self.address_space
    }

    pub fn entry(&self) -> u64 {
        self.entry
    }

    pub fn user_stack_top(&self) -> u64 {
        self.user_stack_top
    }

    #[cfg(feature = "kernel-tests")]
    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
}