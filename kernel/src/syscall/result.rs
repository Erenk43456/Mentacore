pub type SyscallResult = u64;

pub const SYSCALL_OK: SyscallResult = 0;
pub const SYSCALL_ERROR: SyscallResult = u64::MAX;
pub const SYSCALL_ENOSYS: SyscallResult = u64::MAX - 1;
pub const SYSCALL_EFAULT: SyscallResult = u64::MAX - 2;