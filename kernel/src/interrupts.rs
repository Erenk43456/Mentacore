use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::sync::Spinlock;
use crate::memory::physical::PhysicalFrameAllocator;

const COM1: u16 = 0x3F8;

unsafe fn serial_write_byte(byte: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") COM1,
            in("al") byte,
            options(nostack, preserves_flags)
        );
    }
}

fn serial_write(message: &[u8]) {
    for &byte in message {
        unsafe {
            serial_write_byte(byte);
        }
    }
}

fn serial_write_hex(value: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    serial_write(b"0x");

    for i in (0..16).rev() {
        let digit =
            ((value >> (i * 4)) & 0xF) as usize;

        unsafe {
            serial_write_byte(HEX[digit]);
        }
    }
}

#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(
        &mut self,
        handler: unsafe extern "C" fn() -> !,
        selector: u16,
        ist: u8,
    ) {
        let address = handler as u64;

        self.offset_low = address as u16;

        self.selector = selector;

        // Present + interrupt gate + IST index.
        self.options = 0x8E00 | ((ist & 0x07) as u16);

        self.offset_mid = (address >> 16) as u16;

        self.offset_high = (address >> 32) as u32;

        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

const REGISTER_FRAME_SIZE: usize =
    9 * core::mem::size_of::<u64>();

const INTERRUPT_FRAME_WORD_SIZE: usize =
    core::mem::size_of::<u64>();

const CPU_ERROR_CODE_OFFSET: usize =
    REGISTER_FRAME_SIZE;

const CPU_RIP_OFFSET: usize =
    CPU_ERROR_CODE_OFFSET
        + INTERRUPT_FRAME_WORD_SIZE;

const CPU_CS_OFFSET: usize =
    CPU_RIP_OFFSET
        + INTERRUPT_FRAME_WORD_SIZE;

const CPU_RFLAGS_OFFSET: usize =
    CPU_CS_OFFSET
        + INTERRUPT_FRAME_WORD_SIZE;

const CPU_NO_ERROR_RIP_OFFSET: usize =
    CPU_ERROR_CODE_OFFSET;

const _: () = assert!(
    REGISTER_FRAME_SIZE ==
        9 * INTERRUPT_FRAME_WORD_SIZE
);

const _: () = assert!(
    CPU_ERROR_CODE_OFFSET ==
        REGISTER_FRAME_SIZE
);

const _: () = assert!(
    CPU_RIP_OFFSET ==
        CPU_ERROR_CODE_OFFSET
            + INTERRUPT_FRAME_WORD_SIZE
);

const _: () = assert!(
    CPU_CS_OFFSET ==
        CPU_RIP_OFFSET
            + INTERRUPT_FRAME_WORD_SIZE
);

const _: () = assert!(
    CPU_RFLAGS_OFFSET ==
        CPU_CS_OFFSET
            + INTERRUPT_FRAME_WORD_SIZE
);

unsafe fn read_frame_u64(
    register_frame: *const u64,
    offset: usize,
) -> u64 {
    unsafe {
        *((register_frame as *const u8)
            .add(offset)
            as *const u64)
    }
}

static mut IDT: [IdtEntry; 256] =
    [const { IdtEntry::missing() }; 256];

static FRAME_ALLOCATOR: Spinlock<Option<PhysicalFrameAllocator>> =
    Spinlock::new(None);

static TIMER_TICKS: AtomicU64 =
    AtomicU64::new(0);

#[cfg(feature = "kernel-tests")]
static NESTED_EXCEPTION_TEST_ARMED: AtomicU64 =
    AtomicU64::new(0);

pub fn timer_ticks() -> u64 {
    TIMER_TICKS.load(Ordering::Relaxed)
}

pub const LAPIC_TIMER_VECTOR: u8 = 0x40;

#[cfg(feature = "kernel-tests")]
static LAPIC_TIMER_TICKS: AtomicU64 =
    AtomicU64::new(0);

#[cfg(feature = "kernel-tests")]
pub fn lapic_timer_ticks() -> u64 {
    LAPIC_TIMER_TICKS.load(Ordering::Relaxed)
}

unsafe extern "C" {
    fn divide_error_entry() -> !;
    fn invalid_opcode_entry() -> !;
    fn double_fault_entry() -> !;
    fn general_protection_entry() -> !;
    fn page_fault_entry() -> !;
    fn timer_irq_entry() -> !;
    fn lapic_timer_entry() -> !;
}

#[unsafe(no_mangle)]
extern "C" fn divide_error_dispatch(
    register_frame: *const u64,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"       DIVIDE ERROR (#DE)\r\n");
    serial_write(b"================================\r\n");

    unsafe {
        let instruction_pointer =
            read_frame_u64(
                register_frame,
                CPU_NO_ERROR_RIP_OFFSET,
            );

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
    register_frame: *const u64,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b"      INVALID OPCODE (#UD)\r\n");
    serial_write(b"================================\r\n");

    unsafe {
        let instruction_pointer =
            read_frame_u64(
                register_frame,
                CPU_NO_ERROR_RIP_OFFSET,
            );

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
    register_frame: *const u64,
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
            b"NESTED EXCEPTION PATH ARMED\r\n"
        );
    } else {
        serial_write(
            b"NESTED EXCEPTION PATH FAILED\r\n"
        );
    }

    if ist1_ok {
        serial_write(
            b"DOUBLE FAULT IST1 STACK OK\r\n"
        );
    } else {
        serial_write(
            b"DOUBLE FAULT IST1 STACK FAILED\r\n"
        );
    }

    serial_write(b"Error code: ");
    serial_write_hex(error_code);
    serial_write(b"\r\n");

    unsafe {
        let instruction_pointer =
            read_frame_u64(
                register_frame,
                CPU_RIP_OFFSET,
            );

        let code_segment =
            read_frame_u64(
                register_frame,
                CPU_CS_OFFSET,
            );

        let rflags =
            read_frame_u64(
                register_frame,
                CPU_RFLAGS_OFFSET,
            );

        serial_write(b"Instruction pointer: ");
        serial_write_hex(instruction_pointer);
        serial_write(b"\r\n");

        serial_write(b"CS: ");
        serial_write_hex(code_segment);
        serial_write(b"\r\n");

        serial_write(b"RFLAGS: ");
        serial_write_hex(rflags);
        serial_write(b"\r\n");
    }

    #[cfg(feature = "kernel-tests")]
    {
        if ist1_ok && nested_exception_armed {
            let (passed, total) =
                crate::tests::framework::result();

            let passed = passed + 1;

            serial_write(
                b"[TEST] interrupts::double_fault_ist1 ... OK\r\n"
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
                b"[TEST] interrupts::double_fault_ist1 ... FAILED\r\n"
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
    register_frame: *const u64,
    error_code: u64,
) -> ! {
    serial_write(b"\r\n");
    serial_write(b"================================\r\n");
    serial_write(b" GENERAL PROTECTION FAULT (#GP)\r\n");
    serial_write(b"================================\r\n");

    serial_write(b"Error code: ");
    serial_write_hex(error_code);
    serial_write(b"\r\n");

    unsafe {
        let instruction_pointer =
            read_frame_u64(
                register_frame,
                CPU_RIP_OFFSET,
            );

        serial_write(b"Instruction pointer: ");
        serial_write_hex(instruction_pointer);
        serial_write(b"\r\n");
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn timer_irq_dispatch(
    dispatch_rsp: u64,
) {
    #[cfg(feature = "kernel-tests")]
    crate::tests::interrupts::record_timer_dispatch_rsp(
        dispatch_rsp,
    );

    #[cfg(not(feature = "kernel-tests"))]
    let _ = dispatch_rsp;

    TIMER_TICKS.fetch_add(
        1,
        Ordering::Relaxed,
    );

    unsafe {
        crate::hardware::pic::send_eoi(0);
    }
}

#[unsafe(no_mangle)]
extern "C" fn lapic_timer_dispatch(
    dispatch_rsp: u64,
) {
    #[cfg(feature = "kernel-tests")]
    {
        crate::tests::interrupts::record_lapic_timer_dispatch_rsp(
            dispatch_rsp,
        );

        LAPIC_TIMER_TICKS.fetch_add(
            1,
            Ordering::Relaxed,
        );
    }

    #[cfg(not(feature = "kernel-tests"))]
    let _ = dispatch_rsp;

    unsafe {
        crate::hardware::lapic::write_global_eoi();
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

        IDT[14] = IdtEntry::missing();

        core::ptr::read_volatile(
            0x0000_4000_0000_0000 as *const u8
        );
    }

    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
extern "C" fn page_fault_dispatch(
    register_frame: *const u64,
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

    unsafe {
        let instruction_pointer =
            read_frame_u64(
                register_frame,
                CPU_RIP_OFFSET,
            );

        let code_segment =
            read_frame_u64(
                register_frame,
                CPU_CS_OFFSET,
            );

        let rflags =
            read_frame_u64(
                register_frame,
                CPU_RFLAGS_OFFSET,
            );

        serial_write(b"Instruction pointer: ");
        serial_write_hex(instruction_pointer);
        serial_write(b"\r\n");

        serial_write(b"CS: ");
        serial_write_hex(code_segment);
        serial_write(b"\r\n");

        serial_write(b"RFLAGS: ");
        serial_write_hex(rflags);
        serial_write(b"\r\n");
    }

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
        fault_address & !(crate::memory::paging::PAGE_SIZE - 1);

    if page_address < crate::memory::heap::HEAP_START
        || page_address
            >= crate::memory::heap::HEAP_START
                + crate::memory::heap::HEAP_SIZE
    {
        serial_write(b"Page fault outside kernel heap.\r\n");

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
                    b"Physical frame allocator unavailable.\r\n"
                );

                loop {
                    core::hint::spin_loop();
                }
            }
        };

    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,

        None => {
            serial_write(b"Out of physical memory.\r\n");

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
            },
        )
    } {
        Ok(()) => {
            serial_write(b"Page mapped successfully.\r\n");

            unsafe {
                core::arch::asm!(
                    "invlpg [{}]",
                    in(reg) page_address,
                    options(nostack, preserves_flags)
                );
            }

            serial_write(b"TLB invalidated.\r\n");
        }

        Err(()) => {
            serial_write(b"Page mapping failed.\r\n");

            loop {
                core::hint::spin_loop();
            }
        }
    }
}

pub unsafe fn init(
    allocator: PhysicalFrameAllocator,
) {
    {
        let mut guard =
            FRAME_ALLOCATOR.lock_irqsave();

        *guard = Some(allocator);
    }

    unsafe {
        let code_segment: u16;

        asm!(
            "mov {0:x}, cs",
            out(reg) code_segment,
            options(nostack, preserves_flags)
        );

        IDT[0].set_handler(
            divide_error_entry,
            code_segment,
            0,
        );

        IDT[6].set_handler(
            invalid_opcode_entry,
            code_segment,
            0,
        );

        IDT[8].set_handler(
            double_fault_entry,
            code_segment,
            1,
        );

        IDT[13].set_handler(
            general_protection_entry,
            code_segment,
            0,
        );

        IDT[14].set_handler(
            page_fault_entry,
            code_segment,
            0,
        );

        IDT[32].set_handler(
            timer_irq_entry,
            code_segment,
            0,
        );

        IDT[LAPIC_TIMER_VECTOR as usize].set_handler(
            lapic_timer_entry,
            code_segment,
            0,
        );

        let idt_pointer = IdtPointer {
            limit:
                (core::mem::size_of::<IdtEntry>() * 256 - 1)
                    as u16,
            base: core::ptr::addr_of!(IDT) as u64,
        };

        asm!(
            "lidt [{}]",
            in(reg) &idt_pointer,
            options(readonly, nostack, preserves_flags)
        );
    }
}