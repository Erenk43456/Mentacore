use super::context::SyscallContext;
use super::handlers;
use super::numbers::SYS_GET_TID;

pub fn dispatch(context: &SyscallContext) -> u64 {
    match context.rax {
        SYS_GET_TID => handlers::thread::get_tid(),
        _ => u64::MAX,
    }
}