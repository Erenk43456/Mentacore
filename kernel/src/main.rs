#![no_std]
#![no_main]

mod boot;
mod boot_state;
mod cpu;
mod debug;
mod display;
mod interrupts;
mod memory;
mod hardware;
mod sync;
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

    // ---------------------------------------------------------
    // Test runner
    // ---------------------------------------------------------

    tests::write_header();

    let mut test_runner =
        tests::KernelTestRunner::new();

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

    test_runner.run_paging(
        &mut allocator
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

    debug::write(
        b"Preparing LAPIC one-shot timer...\r\n"
    );

    unsafe {
        // Keep the timer masked while programming it.
        lapic.set_lvt_timer(
            hardware::lapic::LAPIC_LVT_MASKED
                | interrupts::LAPIC_TIMER_VECTOR
                    as u32,
        );

        // Divide-by-1.
        lapic.set_timer_divide(
            0x0000_000B
        );

        // Give ourselves plenty of time before
        // the interrupt fires.
        lapic.set_timer_initial_count(
            100_000_000
        );
    }

    let lapic_timer_current =
        unsafe {
            lapic.timer_current_count()
        };

    debug::write(
        b"LAPIC TIMER CURRENT COUNT: "
    );

    debug::write_hex(
        lapic_timer_current as u64
    );

    debug::write(b"\r\n");

    debug::write(
        b"Arming LAPIC one-shot timer...\r\n"
    );

    unsafe {
        // Vector 0x40, one-shot mode, unmasked.
        lapic.set_lvt_timer(
            interrupts::LAPIC_TIMER_VECTOR
                as u32,
        );
    }

    debug::write(
        b"LAPIC TIMER ARMED\r\n"
    );

    debug::write(
        b"Enabling hardware interrupts...\r\n"
    );

    unsafe {
        asm!("sti");
    }

    debug::write(
        b"Hardware interrupts enabled.\r\n"
    );

    // ---------------------------------------------------------
    // Runtime-dependent tests
    // ---------------------------------------------------------
    test_runner.run_heap();

    test_runner.run_interrupts();

    test_runner.run_sync();

    test_runner.run_tsc();

    // ---------------------------------------------------------
    // Final test result
    // ---------------------------------------------------------

    test_runner.finish();

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