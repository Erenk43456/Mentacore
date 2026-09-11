#[repr(C)]
#[derive(Clone, Copy)]
pub struct InterruptContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,

    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,

    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
}

impl InterruptContext {
    pub const fn new(
        rip: u64,
        cs: u64,
        rflags: u64,
    ) -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbp: 0,
            rbx: 0,

            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            rcx: 0,
            rax: 0,

            rip,
            cs,
            rflags,
        }
    }
}