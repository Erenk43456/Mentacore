use mentacore_boot_protocol::BootInfo;

use crate::{
    boot,
    cpu,
    debug,
    interrupts,
    memory,
    scheduler::SchedulerRuntime,
};

mod validation;
mod userspace;

#[cfg(feature = "kernel-tests")]
use crate::tests;

pub fn start(
    boot_info: *const BootInfo,
) -> ! {
    const USERSPACE_PROCESS_ID: u64 = 1;
    const USERSPACE_THREAD_ID: u64 = 10;

    debug::write(
        b"!!! KERNEL _START REACHED !!!\r\n"
    );

    cpu::disable_interrupts();

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

    let boot_info =
        validation::validate_boot_info(
            boot_info
        );

    // ---------------------------------------------------------
    // Memory
    // ---------------------------------------------------------

    let mut frame_allocator = unsafe {
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
        &mut frame_allocator
    );

    // ---------------------------------------------------------
    // Paging
    // ---------------------------------------------------------

    unsafe {
        boot::paging::initialize(
            &mut frame_allocator,
            boot_info,
        );
    }

    #[cfg(feature = "kernel-tests")]
    test_runner.run_paging(
        &mut frame_allocator
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_elf(
        &mut frame_allocator
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_process(
        &mut frame_allocator
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_thread(
        &mut frame_allocator,
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_scheduler(
        &mut frame_allocator,
    );

    // Heap
    unsafe {
        boot::heap::initialize();
    }

    // CPU
    boot::cpu::initialize();

    // LAPIC
    let lapic = unsafe {
        boot::lapic::initialize(
            &mut frame_allocator
        )
    };

    // ---------------------------------------------------------
    // Userspace ELF
    // ---------------------------------------------------------

    let loaded_userspace =
        unsafe {
            userspace::load(
                boot_info,
                &mut frame_allocator,
            )
        };

    // ---------------------------------------------------------
    // Scheduler
    // ---------------------------------------------------------

    if SchedulerRuntime::initialize(
        &mut frame_allocator,
    ).is_err() {
        debug::write(
            b"ERROR: failed to initialize scheduler\r\n",
        );

        loop {
            cpu::halt();
        }
    }

    debug::write(
        b"Scheduler initialized.\r\n"
    );

    #[cfg(feature = "kernel-tests")]
    test_runner.run_syscall();

    #[cfg(feature = "kernel-tests")]
    {
        if !test_runner.prepare_scheduler_timer_preemption(
            &mut frame_allocator,
        ) {
            debug::write(
                b"ERROR: Failed to prepare scheduler timer preemption.\r\n",
            );

            loop {
                cpu::halt();
            }
        }

        if !test_runner.prepare_ring3_transition(
            &mut frame_allocator,
        ) {
            debug::write(
                b"ERROR: Failed to prepare Ring 3 transition test.\r\n",
            );

            loop {
                cpu::halt();
            }
        }
    }

    // ---------------------------------------------------------
    // Userspace process creation
    // ---------------------------------------------------------

    if SchedulerRuntime::create_userspace_process(
        USERSPACE_PROCESS_ID,
        USERSPACE_THREAD_ID,
        loaded_userspace,
        &mut frame_allocator,
    ).is_err() {
        debug::write(
            b"ERROR: failed to create userspace process\r\n",
        );

        loop {
            cpu::halt();
        }
    }

    debug::write(
        b"Userspace process created.\r\n"
    );

    // ---------------------------------------------------------
    // Interrupt system
    // ---------------------------------------------------------

    unsafe {
        boot::interrupts::initialize(
            frame_allocator
        );
    }

    #[cfg(feature = "kernel-tests")]
    test_runner.run_kernel_stack_drop();

    #[cfg(feature = "kernel-tests")]
    test_runner.run_address_space_drop();

    unsafe {
        boot::interrupt_controllers::initialize();
    }

    #[cfg(feature = "kernel-tests")]
    test_runner.run_ring3_user_buffer_validation();

    #[cfg(feature = "kernel-tests")]
    test_runner.run_ring3_transition();

    // ---------------------------------------------------------
    // Display
    // ---------------------------------------------------------

    unsafe {
        boot::display::initialize(
            boot_info
        );
    }

    // ---------------------------------------------------------
    // LAPIC periodic timer
    // ---------------------------------------------------------

    unsafe {
        lapic.arm_timer_periodic(
            interrupts::LAPIC_TIMER_VECTOR,
            100_000_000,
        );
    }

    debug::write(
        b"LAPIC PERIODIC TIMER ARMED\r\n"
    );

    cpu::enable_interrupts();

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
        test_runner.run_scheduler_timer_preemption();
        test_runner.run_double_fault();
        test_runner.finish();
    }

    // ---------------------------------------------------------
    // Userspace launch
    // ---------------------------------------------------------

    debug::write(
        b"Launching userspace...\r\n",
    );

    if SchedulerRuntime::launch_userspace(
        USERSPACE_PROCESS_ID,
        USERSPACE_THREAD_ID,
    ).is_err() {
        debug::write(
            b"ERROR: failed to launch userspace\r\n",
        );

        loop {
            cpu::halt();
        }
    }

    // ---------------------------------------------------------
    // Kernel idle loop
    // ---------------------------------------------------------

    loop {
        cpu::halt();
    }
}