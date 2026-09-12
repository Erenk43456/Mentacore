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
    pub rsp: u64,
    pub ss: u64,
}

impl InterruptContext {
    pub const fn new(
        rip: u64,
        cs: u64,
        rflags: u64,
        rsp: u64,
        ss: u64,
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
            rsp,
            ss,
        }
    }

    pub const fn size() -> usize {
        core::mem::size_of::<Self>()
    }
}

const _: () = assert!(
    core::mem::size_of::<InterruptContext>() == 160
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, r15) == 0
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, r14) == 8
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, r13) == 16
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, r12) == 24
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, rbp) == 32
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, rbx) == 40
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, rip) == 120
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, cs) == 128
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, rflags) == 136
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, rsp) == 144
);

const _: () = assert!(
    core::mem::offset_of!(InterruptContext, ss) == 152
);