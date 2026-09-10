use core::arch::asm;

#[cfg(feature = "kernel-tests")]
use core::sync::atomic::{AtomicU64, Ordering};

use crate::memory::physical::PhysicalFrameAllocator;
use crate::sync::Spinlock;

#[cfg(feature = "kernel-tests")]
use super::idt;

use super::serial::{serial_write, serial_write_hex};
use super::TrapFrame;

static FRAME_ALLOCATOR: Spinlock<Option<PhysicalFrameAllocator>> =
    Spinlock::new(None);

#[cfg(feature = "kernel-tests")]
static NESTED_EXCEPTION_TEST_ARMED: AtomicU64 =
    AtomicU64::new(0);

pub(crate) fn set_frame_allocator(
    allocator: PhysicalFrameAllocator,
) {
    let mut guard = FRAME_ALLOCATOR.lock_irqsave();

    *guard = Some(allocator);
}

#[unsafe(no_mangle)]
extern "C" fn divide_error_dispatch(
    trap_frame: *const TrapFrame,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"       DIVIDE ERROR (#DE)\r\n");
    serial_write(b"================================\r\n");

    unsafe {
        let instruction_pointer =
            (*trap_frame).rip;

        serial_write(b"Instruction pointer: ");
        serial_write_hex(instruction_pointer);
        serial_write(b"\r\n");
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn invalid_opcode_dispatch(
    trap_frame: *const TrapFrame,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"      INVALID OPCODE (#UD)\r\n");
    serial_write(b"================================\r\n");

    unsafe {
        let instruction_pointer =
            (*trap_frame).rip;

        serial_write(b"Instruction pointer: ");
        serial_write_hex(instruction_pointer);
        serial_write(b"\r\n");
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn double_fault_dispatch(
    trap_frame: *const TrapFrame,
    error_code: u64,
    cpu_rsp: u64,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"       DOUBLE FAULT (#DF)\r\n");
    serial_write(b"================================\r\n");

    let (ist1_start, ist1_top) =
        crate::cpu::ist1_stack_range();

    serial_write(b"CPU RSP after IST switch: ");
    serial_write_hex(cpu_rsp);
    serial_write(b"\r\n");

    serial_write(b"IST1 start: ");
    serial_write_hex(ist1_start);
    serial_write(b"\r\n");

    serial_write(b"IST1 top: ");
    serial_write_hex(ist1_top);
    serial_write(b"\r\n");

    let ist1_ok =
        cpu_rsp >= ist1_start
            && cpu_rsp <= ist1_top;

    #[cfg(feature = "kernel-tests")]
    let nested_exception_armed =
        NESTED_EXCEPTION_TEST_ARMED.load(
            Ordering::Relaxed,
        ) == 1;

    #[cfg(feature = "kernel-tests")]
    if nested_exception_armed {
        serial_write(
            b"NESTED EXCEPTION PATH ARMED\r\n",
        );
    } else {
        serial_write(
            b"NESTED EXCEPTION PATH FAILED\r\n",
        );
    }

    if ist1_ok {
        serial_write(
            b"DOUBLE FAULT IST1 STACK OK\r\n",
        );
    } else {
        serial_write(
            b"DOUBLE FAULT IST1 STACK FAILED\r\n",
        );
    }

    let instruction_pointer =
        unsafe { (*trap_frame).rip };

    let code_segment =
        unsafe { (*trap_frame).cs };

    let rflags =
        unsafe { (*trap_frame).rflags };

    serial_write(b"Error code: ");
    serial_write_hex(error_code);
    serial_write(b"\r\n");

    serial_write(b"Instruction pointer: ");
    serial_write_hex(instruction_pointer);
    serial_write(b"\r\n");

    serial_write(b"CS: ");
    serial_write_hex(code_segment);
    serial_write(b"\r\n");

    serial_write(b"RFLAGS: ");
    serial_write_hex(rflags);
    serial_write(b"\r\n");

    #[cfg(feature = "kernel-tests")]
    {
        if ist1_ok && nested_exception_armed {
            let (passed, total) =
                crate::tests::framework::result();

            let passed = passed + 1;

            serial_write(
                b"[TEST] interrupts::double_fault_ist1 ... OK\r\n",
            );

            serial_write(b"\r\nRESULT: ");
            crate::tests::framework::write_usize(passed);
            serial_write(b"/");
            crate::tests::framework::write_usize(total);
            serial_write(b" TESTS PASSED\r\n");

            if passed == total {
                serial_write(b"ALL TESTS PASSED\r\n");
            } else {
                serial_write(b"TESTS FAILED\r\n");
            }
        } else {
            let (passed, total) =
                crate::tests::framework::result();

            serial_write(
                b"[TEST] interrupts::double_fault_ist1 ... FAILED\r\n",
            );

            serial_write(b"\r\nRESULT: ");
            crate::tests::framework::write_usize(passed);
            serial_write(b"/");
            crate::tests::framework::write_usize(total);
            serial_write(b" TESTS PASSED\r\n");

            serial_write(b"TESTS FAILED\r\n");
        }
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn general_protection_dispatch(
    trap_frame: *const TrapFrame,
    error_code: u64,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b" GENERAL PROTECTION FAULT (#GP)\r\n");
    serial_write(b"================================\r\n");

    serial_write(b"Error code: ");
    serial_write_hex(error_code);
    serial_write(b"\r\n");

    let instruction_pointer =
        unsafe { (*trap_frame).rip };

    serial_write(b"Instruction pointer: ");
    serial_write_hex(instruction_pointer);
    serial_write(b"\r\n");

    loop {
        core::hint::spin_loop();
    }
}

#[cfg(feature = "kernel-tests")]
pub fn trigger_double_fault_test() -> ! {
    NESTED_EXCEPTION_TEST_ARMED.store(
        1,
        Ordering::Relaxed,
    );

    unsafe {
        core::arch::asm!(
            "cli",
            options(nostack)
        );

        idt::clear_handler(
            idt::PAGE_FAULT_VECTOR,
        );

        core::ptr::read_volatile(
            0x0000_5000_0000_0000 as *const u8,
        );
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn page_fault_dispatch(
    trap_frame: *const TrapFrame,
    error_code: u64,
) {
    let fault_address: u64;

    unsafe {
        asm!(
            "mov {}, cr2",
            out(reg) fault_address,
            options(nostack, preserves_flags)
        );
    }

    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"           PAGE FAULT\r\n");
    serial_write(b"================================\r\n");

    serial_write(b"Fault address: ");
    serial_write_hex(fault_address);
    serial_write(b"\r\n");

    serial_write(b"Error code: ");
    serial_write_hex(error_code);
    serial_write(b"\r\n");

    let instruction_pointer =
        unsafe { (*trap_frame).rip };

    let code_segment =
        unsafe { (*trap_frame).cs };

    let rflags =
        unsafe { (*trap_frame).rflags };

    serial_write(b"Instruction pointer: ");
    serial_write_hex(instruction_pointer);
    serial_write(b"\r\n");

    serial_write(b"CS: ");
    serial_write_hex(code_segment);
    serial_write(b"\r\n");

    serial_write(b"RFLAGS: ");
    serial_write_hex(rflags);
    serial_write(b"\r\n");

    // Bit 0 = 1:
    // Page is present, but access was denied.
    // This is NOT a demand-paging fault.
    if error_code & 1 != 0 {
        serial_write(b"Protection fault.\r\n");

        loop {
            core::hint::spin_loop();
        }
    }

    serial_write(b"Page fault handler reached.\r\n");

    let page_address =
        fault_address
            & !(crate::memory::paging::PAGE_SIZE - 1);

    if page_address < crate::memory::heap::HEAP_START
        || page_address
            >= crate::memory::heap::HEAP_START
                + crate::memory::heap::HEAP_SIZE
    {
        serial_write(
            b"Page fault outside kernel heap.\r\n",
        );

        loop {
            core::hint::spin_loop();
        }
    }

    let mut allocator_guard =
        FRAME_ALLOCATOR.lock_irqsave();

    let allocator =
        match allocator_guard.as_mut() {
            Some(allocator) => allocator,

            None => {
                serial_write(
                    b"Physical frame allocator unavailable.\r\n",
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,

        None => {
            serial_write(
                b"Out of physical memory.\r\n",
            );

            loop {
                core::hint::spin_loop();
            }
        }
    };

    serial_write(b"Allocated frame: ");
    serial_write_hex(frame.start_address);
    serial_write(b"\r\n");

    let pml4 = unsafe {
        crate::memory::paging::current_pml4()
    };

    match unsafe {
        crate::memory::paging::map_page(
            pml4,
            allocator,
            page_address,
            frame.start_address,
            crate::memory::paging::PageFlags {
                writable: true,
                cache_disable: false,
                user: false,
            },
        )
    } {
        Ok(()) => {
            serial_write(
                b"Page mapped successfully.\r\n",
            );

            unsafe {
                core::arch::asm!(
                    "invlpg [{}]",
                    in(reg) page_address,
                    options(
                        nostack,
                        preserves_flags
                    )
                );
            }

            serial_write(b"TLB invalidated.\r\n");
        }

        Err(()) => {
            serial_write(
                b"Page mapping failed.\r\n",
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }
}