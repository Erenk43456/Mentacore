use core::arch::asm;
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
    ) {
        let address = handler as u64;

        self.offset_low = address as u16;

        self.selector = selector;

        // Present + interrupt gate
        self.options = 0x8E00;

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

static mut IDT: [IdtEntry; 256] =
    [const { IdtEntry::missing() }; 256];

static mut FRAME_ALLOCATOR: *mut () =
    core::ptr::null_mut();

#[unsafe(naked)]
unsafe extern "C" fn page_fault_entry() -> ! {
    core::arch::naked_asm!(
        "cli",

        // Save registers.
        "push rax",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",

        // Arguments:
        // RDI = register frame
        // RSI = CPU error code
        "mov rdi, rsp",
        "mov rsi, [rsp + 72]",

        "call {handler}",

        // Restore registers.
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rax",

        // Remove CPU-pushed page-fault error code.
        "add rsp, 8",
        
        // Return to the faulting instruction.
        "iretq",

        handler = sym page_fault_dispatch,
    );
}

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
            *((register_frame as *const u8).add(80) as *const u64);

        let code_segment =
            *((register_frame as *const u8).add(88) as *const u64);

        let rflags =
            *((register_frame as *const u8).add(96) as *const u64);

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

    let allocator = unsafe {
        &mut *(FRAME_ALLOCATOR as *mut PhysicalFrameAllocator)
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
    allocator: &mut PhysicalFrameAllocator,
) {
    unsafe {
        FRAME_ALLOCATOR =
            allocator as *mut PhysicalFrameAllocator as *mut ();

        let code_segment: u16;

        asm!(
            "mov {0:x}, cs",
            out(reg) code_segment,
            options(nostack, preserves_flags)
        );

        IDT[14].set_handler(
            page_fault_entry,
            code_segment,
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