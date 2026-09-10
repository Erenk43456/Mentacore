#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(super) struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    pub(super) const fn code() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0x9A,
            granularity: 0xAF,
            base_high: 0,
        }
    }

    pub(super) const fn data() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0x92,
            granularity: 0xAF,
            base_high: 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(super) struct GdtPointer {
    pub(super) limit: u16,
    pub(super) base: u64,
}

pub(super) const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub(super) const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub(super) const TSS_SELECTOR: u16 = 0x18;

#[repr(align(16))]
pub(super) struct Gdt {
    pub(super) entries: [u64; 5],
}

impl Gdt {
    pub(super) const fn new() -> Self {
        Self {
            entries: [0; 5],
        }
    }
}

pub(super) static mut GDT: Gdt = Gdt::new();

pub(super) unsafe fn write_gdt_entry(
    gdt: *mut Gdt,
    index: usize,
    entry: GdtEntry,
) {
    let value =
        (entry.limit_low as u64)
        | ((entry.base_low as u64) << 16)
        | ((entry.base_middle as u64) << 32)
        | ((entry.access as u64) << 40)
        | ((entry.granularity as u64) << 48)
        | ((entry.base_high as u64) << 56);

    unsafe {
        (*gdt).entries[index] = value;
    }
}

pub(super) unsafe fn write_tss_descriptor(
    gdt: *mut Gdt,
    index: usize,
    descriptor: TssDescriptor,
) {
    let low =
        (descriptor.limit_low as u64)
        | ((descriptor.base_low as u64) << 16)
        | ((descriptor.base_middle as u64) << 32)
        | ((descriptor.access as u64) << 40)
        | ((descriptor.granularity as u64) << 48)
        | ((descriptor.base_high as u64) << 56);

    unsafe {
        (*gdt).entries[index] = low;

        (*gdt).entries[index + 1] =
            (descriptor.base_upper as u64)
            | ((descriptor.reserved as u64) << 32);
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(super) struct TssDescriptor {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
    base_upper: u32,
    reserved: u32,
}

impl TssDescriptor {
    pub(super) fn new(tss: *const super::tss::Tss) -> Self {
        let base = tss as u64;

        let limit =
            (core::mem::size_of::<super::tss::Tss>() - 1) as u32;

        Self {
            limit_low: limit as u16,
            base_low: base as u16,
            base_middle: (base >> 16) as u8,
            access: 0x89,
            granularity: ((limit >> 16) & 0x0F) as u8,
            base_high: (base >> 24) as u8,
            base_upper: (base >> 32) as u32,
            reserved: 0,
        }
    }
}

unsafe extern "C" {
    fn load_gdt_and_segments(
        gdt_pointer: *const GdtPointer,
    );
}

pub(super) unsafe fn load(gdt: *const Gdt) {
    let gdt_pointer = GdtPointer {
        limit: (core::mem::size_of::<Gdt>() - 1) as u16,
        base: gdt as u64,
    };

    unsafe {
        load_gdt_and_segments(&gdt_pointer);

        core::arch::asm!(
            "mov ax, {selector}",
            "ltr ax",
            selector = const TSS_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
}