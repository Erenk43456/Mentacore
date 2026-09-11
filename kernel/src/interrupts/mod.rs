use crate::memory::physical::PhysicalFrameAllocator;

mod exceptions;
mod idt;
mod serial;
mod timer;

pub use timer::timer_ticks;
pub use timer::LAPIC_TIMER_VECTOR;

#[cfg(feature = "kernel-tests")]
pub use exceptions::trigger_double_fault_test;

#[repr(C)]
pub struct TrapFrame {
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,

    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
}

const _: () = assert!(
    core::mem::size_of::<TrapFrame>()
        == 13 * core::mem::size_of::<u64>()
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, r11)
        == 0
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, r10)
        == 8
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, r9)
        == 16
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, r8)
        == 24
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rdi)
        == 32
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rsi)
        == 40
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rdx)
        == 48
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rcx)
        == 56
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rax)
        == 64
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, error_code)
        == 72
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rip)
        == 80
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, cs)
        == 88
);

const _: () = assert!(
    core::mem::offset_of!(TrapFrame, rflags)
        == 96
);

#[cfg(feature = "kernel-tests")]
pub fn lapic_timer_ticks() -> u64 {
    timer::lapic_timer_ticks()
}

#[cfg(feature = "kernel-tests")]
pub fn validate_interrupt_context_layout() -> bool {
    timer::validate_interrupt_context_layout()
}

pub unsafe fn init(
    allocator: PhysicalFrameAllocator,
) {
    exceptions::set_frame_allocator(
        allocator,
    );

    unsafe {
        idt::init();
    }
}