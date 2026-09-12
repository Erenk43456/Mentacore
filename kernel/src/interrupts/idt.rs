use core::arch::asm;

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

    fn set_user_handler(
        &mut self,
        handler: unsafe extern "C" fn() -> !,
        selector: u16,
    ) {
        let address = handler as u64;

        self.offset_low = address as u16;
        self.selector = selector;

        // Present + interrupt gate + DPL 3.
        self.options = 0x8E00 | (3 << 13);

        self.offset_mid =
            (address >> 16) as u16;

        self.offset_high =
            (address >> 32) as u32;

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

unsafe extern "C" {
    fn divide_error_entry() -> !;
    fn invalid_opcode_entry() -> !;
    fn double_fault_entry() -> !;
    fn general_protection_entry() -> !;
    fn page_fault_entry() -> !;
    fn timer_irq_entry() -> !;
    fn lapic_timer_entry() -> !;
    #[cfg(feature = "kernel-tests")]
    fn user_privilege_interrupt_entry() -> !;
}

#[cfg(feature = "kernel-tests")]
pub(crate) const USER_TEST_VECTOR: usize = 0x80;

pub(crate) const PAGE_FAULT_VECTOR: usize = 14;

#[cfg(feature = "kernel-tests")]
pub(crate) unsafe fn clear_handler(vector: usize) {
    IDT[vector] = IdtEntry::missing();
}

pub(crate) unsafe fn init() {
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

        IDT[PAGE_FAULT_VECTOR].set_handler(
            page_fault_entry,
            code_segment,
            0,
        );

        IDT[32].set_handler(
            timer_irq_entry,
            code_segment,
            0,
        );

        IDT[super::timer::LAPIC_TIMER_VECTOR as usize]
            .set_handler(
                lapic_timer_entry,
                code_segment,
                0,
            );
            
        #[cfg(feature = "kernel-tests")]
        IDT[USER_TEST_VECTOR].set_user_handler(
            user_privilege_interrupt_entry,
            code_segment,
        );

        let idt_pointer = IdtPointer {
            limit: (core::mem::size_of::<IdtEntry>() * 256 - 1)
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