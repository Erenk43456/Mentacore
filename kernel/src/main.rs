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
#[cfg(feature = "kernel-tests")]
mod tests;

use core::arch::asm;
use core::panic::PanicInfo;

use mentacore_boot_protocol::BootInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start(
    boot_info: *const BootInfo,
) -> ! {
    debug::write(
        b"!!! KERNEL _START REACHED !!!\r\n"
    );

    unsafe {
        asm!("cli");
    }

    debug::write(b"\r\n");
    debug::write(
        b"================================\r\n",
    );
    debug::write(
        b"       MENTACORE KERNEL\r\n",
    );
    debug::write(
        b"================================\r\n",
    );
    debug::write(b"\r\n");

    if boot_info.is_null() {
        debug::write(
            b"ERROR: BootInfo is NULL\r\n",
        );

        loop {
            core::hint::spin_loop();
        }
    }

    debug::write(
        b"BootInfo received.\r\n"
    );

    let boot_info =
        unsafe { &*boot_info };

    // ---------------------------------------------------------
    // Memory
    // ---------------------------------------------------------

    let mut allocator = unsafe {
        boot::memory::initialize(
            boot_info
        )
    };

    #[cfg(feature = "kernel-tests")]
    tests::write_header();

    #[cfg(feature = "kernel-tests")]
    let mut test_runner =
        tests::KernelTestRunner::new();

    #[cfg(feature = "kernel-tests")]
    test_runner.run_physical(
        &mut allocator
    );

    // ---------------------------------------------------------
    // Paging
    // ---------------------------------------------------------

    unsafe {
        boot::paging::initialize(
            &mut allocator,
            boot_info,
        );
    }

    #[cfg(feature = "kernel-tests")]
    test_runner.run_paging(
        &mut allocator
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_process(
        &mut allocator
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_thread(
        &mut allocator,
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_scheduler(
        &mut allocator,
    );

    // ---------------------------------------------------------
    // Heap
    // ---------------------------------------------------------

    unsafe {
        boot::heap::initialize();
    }

    // ---------------------------------------------------------
    // CPU
    // ---------------------------------------------------------

    boot::cpu::initialize();

    // ---------------------------------------------------------
    // Scheduler
    // ---------------------------------------------------------

    scheduler::SchedulerRuntime::initialize(
        &mut allocator,
    )
    .expect("failed to initialize scheduler");

    debug::write(
        b"Scheduler initialized.\r\n"
    );

    // ---------------------------------------------------------
    // LAPIC
    // ---------------------------------------------------------

    let lapic = unsafe {
        boot::lapic::initialize(
            &mut allocator
        )
    };

    // ---------------------------------------------------------
    // Interrupt system
    // ---------------------------------------------------------

    unsafe {
        boot::interrupts::initialize(
            allocator
        );
    }

    unsafe {
        boot::interrupt_controllers::initialize();
    }

    // ---------------------------------------------------------
    // Display
    // ---------------------------------------------------------

    unsafe {
        boot::display::initialize(
            boot_info
        );
    }

    // ---------------------------------------------------------
    // LAPIC one-shot timer
    // ---------------------------------------------------------

    unsafe {
        lapic.arm_timer_oneshot(
            interrupts::LAPIC_TIMER_VECTOR,
            100_000_000,
        );
    }

    debug::write(
        b"LAPIC TIMER ARMED\r\n"
    );

    unsafe {
        asm!("sti");
    }

    debug::write(
        b"Hardware interrupts enabled.\r\n"
    );

    // ---------------------------------------------------------
    // Kernel tests
    // ---------------------------------------------------------

    #[cfg(feature = "kernel-tests")]
    {
        test_runner.run_heap();
        test_runner.run_sync();
        test_runner.run_tsc();
        test_runner.run_cpu();
        test_runner.run_interrupts(&lapic);
        test_runner.finish();
    }

    // ---------------------------------------------------------
    // Kernel idle loop
    // ---------------------------------------------------------

    loop {
        cpu::halt();
    }
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