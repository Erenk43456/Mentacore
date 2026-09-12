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
}