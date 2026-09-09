use crate::cpu;

const IA32_APIC_BASE_MSR: u32 = 0x1B;

const APIC_BASE_MASK: u64 = 0xFFFF_FFFF_FFFF_F000;
const APIC_GLOBAL_ENABLE: u64 = 1 << 11;

pub struct Lapic {
    base: u64,
    msr_value: u64,
}

impl Lapic {
    pub fn discover() -> Option<Self> {
        let msr_value = cpu::read_msr(IA32_APIC_BASE_MSR);

        if (msr_value & APIC_GLOBAL_ENABLE) == 0 {
            return None;
        }

        let base = msr_value & APIC_BASE_MASK;

        if base == 0 {
            return None;
        }

        Some(Self {
            base,
            msr_value,
        })
    }

    pub fn base(&self) -> u64 {
        self.base
    }

    pub fn msr_value(&self) -> u64 {
        self.msr_value
    }
}