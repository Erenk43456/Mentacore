use core::ptr::{read_volatile, write_volatile};

use crate::cpu;

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const APIC_BASE_MASK: u64 = 0xFFFF_FFFF_FFFF_F000;
const APIC_GLOBAL_ENABLE: u64 = 1 << 11;

pub const LAPIC_VIRTUAL_BASE: u64 =
    0xFFFF_A000_0000_0000;

pub const LAPIC_ID_OFFSET: u64 = 0x020;
pub const LAPIC_VERSION_OFFSET: u64 = 0x030;
pub const LAPIC_EOI_OFFSET: u64 = 0x0B0;

pub const LAPIC_SVR_OFFSET: u64 = 0x0F0;
pub const LAPIC_SVR_ENABLE: u32 = 1 << 8;

pub const LAPIC_LVT_TIMER_OFFSET: u64 = 0x320;
pub const LAPIC_LVT_ERROR_OFFSET: u64 = 0x370;

pub const LAPIC_TIMER_INITIAL_COUNT_OFFSET: u64 = 0x380;
pub const LAPIC_TIMER_CURRENT_COUNT_OFFSET: u64 = 0x390;
pub const LAPIC_TIMER_DIVIDE_OFFSET: u64 = 0x3E0;

pub const LAPIC_LVT_TIMER_PERIODIC: u32 = 1 << 17;
pub const LAPIC_LVT_MASKED: u32 = 1 << 16;

pub struct Lapic {
    physical_base: u64,
    virtual_base: u64,
    msr_value: u64,
}

impl Lapic {
    pub fn discover() -> Option<Self> {
        let msr_value =
            cpu::read_msr(IA32_APIC_BASE_MSR);

        if (msr_value & APIC_GLOBAL_ENABLE) == 0 {
            return None;
        }

        let physical_base =
            msr_value & APIC_BASE_MASK;

        if physical_base == 0 {
            return None;
        }

        Some(Self {
            physical_base,
            virtual_base: 0,
            msr_value,
        })
    }

    pub fn set_virtual_base(
        &mut self,
        virtual_base: u64,
    ) {
        self.virtual_base = virtual_base;
    }

    pub fn physical_base(&self) -> u64 {
        self.physical_base
    }

    pub fn virtual_base(&self) -> u64 {
        self.virtual_base
    }

    pub fn msr_value(&self) -> u64 {
        self.msr_value
    }

    pub unsafe fn read_u32(
        &self,
        offset: u64,
    ) -> u32 {
        let address =
            (self.virtual_base + offset)
                as *const u32;

        unsafe {
            read_volatile(address)
        }
    }

    pub unsafe fn write_u32(
        &self,
        offset: u64,
        value: u32,
    ) {
        let address =
            (self.virtual_base + offset)
                as *mut u32;

        unsafe {
            write_volatile(address, value);
        }
    }

    pub unsafe fn id(&self) -> u32 {
        unsafe {
            self.read_u32(LAPIC_ID_OFFSET)
        }
    }

    pub unsafe fn version(&self) -> u32 {
        unsafe {
            self.read_u32(
                LAPIC_VERSION_OFFSET
            )
        }
    }

    pub unsafe fn write_eoi(&self) {
        unsafe {
            self.write_u32(
                LAPIC_EOI_OFFSET,
                0,
            );
        }
    }

    pub unsafe fn svr(&self) -> u32 {
        unsafe {
            self.read_u32(LAPIC_SVR_OFFSET)
        }
    }

    pub unsafe fn set_svr(
        &self,
        value: u32,
    ) {
        unsafe {
            self.write_u32(
                LAPIC_SVR_OFFSET,
                value,
            );
        }
    }

    pub unsafe fn lvt_timer(&self) -> u32 {
        unsafe {
            self.read_u32(
                LAPIC_LVT_TIMER_OFFSET
            )
        }
    }

    pub unsafe fn lvt_error(&self) -> u32 {
        unsafe {
            self.read_u32(
                LAPIC_LVT_ERROR_OFFSET
            )
        }
    }

    pub unsafe fn timer_initial_count(
        &self,
    ) -> u32 {
        unsafe {
            self.read_u32(
                LAPIC_TIMER_INITIAL_COUNT_OFFSET
            )
        }
    }

    pub unsafe fn set_timer_initial_count(
        &self,
        value: u32,
    ) {
        unsafe {
            self.write_u32(
                LAPIC_TIMER_INITIAL_COUNT_OFFSET,
                value,
            );
        }
    }

    pub unsafe fn timer_current_count(
        &self,
    ) -> u32 {
        unsafe {
            self.read_u32(
                LAPIC_TIMER_CURRENT_COUNT_OFFSET
            )
        }
    }

    pub unsafe fn timer_divide(&self) -> u32 {
        unsafe {
            self.read_u32(
                LAPIC_TIMER_DIVIDE_OFFSET
            )
        }
    }

    pub unsafe fn set_timer_divide(
        &self,
        value: u32,
    ) {
        unsafe {
            self.write_u32(
                LAPIC_TIMER_DIVIDE_OFFSET,
                value,
            );
        }
    }
}

pub unsafe fn write_global_eoi() {
    let address =
        (LAPIC_VIRTUAL_BASE + LAPIC_EOI_OFFSET)
            as *mut u32;

    unsafe {
        write_volatile(address, 0);
    }
}