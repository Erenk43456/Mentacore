#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

const SYS_USER_START: u64 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            "int 0x80",
            in("rax") SYS_USER_START,
            options(nostack)
        );
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}