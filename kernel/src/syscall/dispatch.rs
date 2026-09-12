use super::context::SyscallContext;
use super::handlers;
use super::numbers::SYS_GET_TID;
use super::result::{
    SyscallResult,
    SYSCALL_ENOSYS,
};

pub fn dispatch(context: &SyscallContext) -> SyscallResult {
    match context.number() {
        SYS_GET_TID => handlers::thread::get_tid(),
        _ => SYSCALL_ENOSYS,
    }
}