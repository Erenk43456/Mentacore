mod manager;
mod process;

pub use manager::{
    ProcessManager,
    MAX_PROCESSES,
};

pub use process::{
    Process,
    ProcessId,
    ProcessState,
};