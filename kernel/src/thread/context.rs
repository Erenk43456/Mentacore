#[repr(C)]
#[derive(Clone, Copy)]
pub struct KernelContext {
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,

    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl KernelContext {
    pub const fn new(
        rsp: u64,
        rip: u64,
    ) -> Self {
        Self {
            rsp,
            rip,
            rflags: super::INITIAL_RFLAGS,

            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }

    #[cfg(feature = "kernel-tests")]
    pub fn rsp(&self) -> u64 {
        self.rsp
    }

    pub fn rip(&self) -> u64 {
        self.rip
    }

    pub fn rflags(&self) -> u64 {
        self.rflags
    }
}

unsafe extern "C" {
    #[cfg(feature = "kernel-tests")]
    pub unsafe fn context_switch(
        current: *mut KernelContext,
        next: *const KernelContext,
    );

    pub fn interrupt_context_switch(
        current_rsp: *mut u64,
        next: *const u64,
    ) -> !;

    pub fn interrupt_context_switch_to_address_space(
        current_rsp: *mut u64,
        next: *const u64,
        next_pml4: u64,
    ) -> !;
}