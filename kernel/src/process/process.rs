use crate::memory::paging::AddressSpace;

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
}

impl Process {
    pub unsafe fn new(
        pid: ProcessId,
        allocator: &mut crate::memory::physical::PhysicalFrameAllocator,
    ) -> Result<Self, ()> {
        let address_space =
            unsafe { AddressSpace::new(allocator)? };

        Ok(Self {
            pid,
            state: ProcessState::Ready,
            address_space,
        })
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

    pub fn set_state(&mut self, state: ProcessState) {
        self.state = state;
    }
}