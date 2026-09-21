pub mod context;
pub mod dispatch;
pub mod handlers;
pub mod numbers;
pub mod result;
pub mod user;

#[cfg(feature = "kernel-tests")]
pub use context::SyscallContext;

#[cfg(feature = "kernel-tests")]
pub use dispatch::dispatch;

#[cfg(feature = "kernel-tests")]
pub use numbers::SYS_GET_TID;

#[cfg(feature = "kernel-tests")]
pub use result::SYSCALL_ENOSYS;

#[cfg(feature = "kernel-tests")]
pub use user::{
    validate_user_address,
    validate_user_buffer,
};