mod context;
mod interrupt_context;
mod manager;
mod stack;
mod thread;

pub(crate) const INITIAL_RFLAGS: u64 = 0x202;

pub use context::{
    interrupt_context_switch,
    interrupt_context_switch_to_address_space,
    KernelContext,
};

#[cfg(feature = "kernel-tests")]
pub use context::context_switch;

pub use interrupt_context::{
    InterruptContext,
    KernelInterruptContext,
};

pub use manager::ThreadManager;

pub use stack::KernelStack;

#[cfg(feature = "kernel-tests")]
pub use stack::{
    KERNEL_STACK_ALIGNMENT,
    KERNEL_STACK_PAGES,
    KERNEL_STACK_SIZE,
};

pub use thread::{
    Thread,
    ThreadEntry,
    ThreadState,
    ThreadId,
};