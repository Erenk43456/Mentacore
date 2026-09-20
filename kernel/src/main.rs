#![no_std]
#![no_main]

mod boot;
mod boot_state;
mod cpu;
mod process;
mod thread;
mod scheduler;
mod debug;
mod display;
mod interrupts;
mod memory;
mod hardware;
mod sync;
mod kernel_init;
#[cfg(feature = "kernel-tests")]
mod tests;
mod syscall;

use core::panic::PanicInfo;

use mentacore_boot_protocol::BootInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start(
    boot_info: *const BootInfo,
) -> ! {
    kernel_init::start(boot_info)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    debug::write(
        b"MENTACORE KERNEL PANIC\r\n"
    );

    loop {
        core::hint::spin_loop();
    }
}