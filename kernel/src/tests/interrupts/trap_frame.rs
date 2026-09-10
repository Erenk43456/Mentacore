use crate::interrupts::TrapFrame;

pub(super) fn test_trap_frame_layout() -> bool {
    if core::mem::size_of::<TrapFrame>()
        != 13 * core::mem::size_of::<u64>()
    {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r11) != 0 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r10) != 8 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r9) != 16 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, r8) != 24 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rdi) != 32 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rsi) != 40 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rdx) != 48 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rcx) != 56 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rax) != 64 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, error_code) != 72 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rip) != 80 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, cs) != 88 {
        return false;
    }

    if core::mem::offset_of!(TrapFrame, rflags) != 96 {
        return false;
    }

    true
}