use super::context::SyscallContext;
use super::handlers;
use super::numbers::{
    SYS_GET_TID,
    SYS_USER_START,
};
use super::result::{
    SyscallResult,
    SYSCALL_ENOSYS,
};

pub fn dispatch(context: &SyscallContext) -> SyscallResult {
    match context.number() {
        SYS_GET_TID => handlers::thread::get_tid(),
        SYS_USER_START => handlers::thread::user_start(),
        _ => SYSCALL_ENOSYS,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn syscall_interrupt_dispatch(
    saved_registers: *mut u64,
    _cpu_frame: *mut u64,
    _current_rsp: u64,
) {
    let syscall_number = unsafe {
        *saved_registers.add(14)
    };

    let context = SyscallContext::new(
        syscall_number,
        unsafe { *saved_registers.add(10) },
        unsafe { *saved_registers.add(11) },
        unsafe { *saved_registers.add(12) },
        unsafe { *saved_registers.add(7) },
        unsafe { *saved_registers.add(9) },
        unsafe { *saved_registers.add(8) },
    );

    let result = dispatch(&context);

    unsafe {
        *saved_registers.add(14) = result;
    }
}