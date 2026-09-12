pub struct SyscallContext {
    pub rax: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub r10: u64,
    pub r8: u64,
    pub r9: u64,
}

impl SyscallContext {
    pub const fn new(
        rax: u64,
        rdi: u64,
        rsi: u64,
        rdx: u64,
        r10: u64,
        r8: u64,
        r9: u64,
    ) -> Self {
        Self {
            rax,
            rdi,
            rsi,
            rdx,
            r10,
            r8,
            r9,
        }
    }

    pub const fn number(&self) -> u64 {
        self.rax
    }

    pub const fn arg0(&self) -> u64 {
        self.rdi
    }

    pub const fn arg1(&self) -> u64 {
        self.rsi
    }

    pub const fn arg2(&self) -> u64 {
        self.rdx
    }

    pub const fn arg3(&self) -> u64 {
        self.r10
    }

    pub const fn arg4(&self) -> u64 {
        self.r8
    }

    pub const fn arg5(&self) -> u64 {
        self.r9
    }
}