use core::arch::naked_asm;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }

    const fn code() -> Self {
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access: 0x9A,
            granularity: 0xAF,
            base_high: 0,
        }
    }

    const fn data() -> Self {
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
struct Tss {
    reserved_0: u32,

    rsp0_low: u32,
    rsp0_high: u32,

    rsp1_low: u32,
    rsp1_high: u32,

    rsp2_low: u32,
    rsp2_high: u32,

    reserved_1: u64,

    ist1_low: u32,
    ist1_high: u32,

    ist2_low: u32,
    ist2_high: u32,

    ist3_low: u32,
    ist3_high: u32,

    ist4_low: u32,
    ist4_high: u32,

    ist5_low: u32,
    ist5_high: u32,

    ist6_low: u32,
    ist6_high: u32,

    ist7_low: u32,
    ist7_high: u32,

    reserved_2: u64,
    reserved_3: u16,

    iomap_base: u16,
}

impl Tss {
    const fn new() -> Self {
        Self {
            reserved_0: 0,

            rsp0_low: 0,
            rsp0_high: 0,

            rsp1_low: 0,
            rsp1_high: 0,

            rsp2_low: 0,
            rsp2_high: 0,

            reserved_1: 0,

            ist1_low: 0,
            ist1_high: 0,

            ist2_low: 0,
            ist2_high: 0,

            ist3_low: 0,
            ist3_high: 0,

            ist4_low: 0,
            ist4_high: 0,

            ist5_low: 0,
            ist5_high: 0,

            ist6_low: 0,
            ist6_high: 0,

            ist7_low: 0,
            ist7_high: 0,

            reserved_2: 0,
            reserved_3: 0,

            iomap_base: core::mem::size_of::<Self>() as u16,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct TssDescriptor {
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
    fn new(tss: *const Tss) -> Self {
        let base = tss as u64;
        let limit =
            (core::mem::size_of::<Tss>() - 1) as u32;

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

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GdtPointer {
    limit: u16,
    base: u64,
}

const KERNEL_CODE_SELECTOR: u16 = 0x08;
const KERNEL_DATA_SELECTOR: u16 = 0x10;
const TSS_SELECTOR: u16 = 0x18;

#[repr(align(16))]
struct Gdt {
    entries: [u64; 5],
}

static mut TSS: Tss = Tss::new();

static mut GDT: Gdt = Gdt {
    entries: [0; 5],
};

fn write_gdt_entry(index: usize, entry: GdtEntry) {
    let value =
        (entry.limit_low as u64)
        | ((entry.base_low as u64) << 16)
        | ((entry.base_middle as u64) << 32)
        | ((entry.access as u64) << 40)
        | ((entry.granularity as u64) << 48)
        | ((entry.base_high as u64) << 56);

    unsafe {
        GDT.entries[index] = value;
    }
}

fn write_tss_descriptor(index: usize, descriptor: TssDescriptor) {
    let low =
        (descriptor.limit_low as u64)
        | ((descriptor.base_low as u64) << 16)
        | ((descriptor.base_middle as u64) << 32)
        | ((descriptor.access as u64) << 40)
        | ((descriptor.granularity as u64) << 48)
        | ((descriptor.base_high as u64) << 56);

    unsafe {
        GDT.entries[index] = low;
        GDT.entries[index + 1] =
            (descriptor.base_upper as u64)
            | ((descriptor.reserved as u64) << 32);
    }
}

#[unsafe(naked)]
unsafe extern "C" fn load_gdt_and_segments(
    gdt_pointer: *const GdtPointer,
) {
    naked_asm!(
        "lgdt [rdi]",

        // Reload CS with our kernel code segment.
        "push 0x08",
        "lea rax, [rip + 1f]",
        "push rax",
        "retfq",

        "1:",

        // Reload data segments.
        "mov ax, 0x10",
        "mov ds, ax",
        "mov es, ax",
        "mov ss, ax",

        "ret",
    );
}

unsafe fn load_tss() {
    unsafe {
        core::arch::asm!(
            "mov ax, {selector}",
            "ltr ax",
            selector = const TSS_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
}

pub fn init() {
    unsafe {
        GDT.entries = [0; 5];
    }

    write_gdt_entry(
        1,
        GdtEntry::code(),
    );

    write_gdt_entry(
        2,
        GdtEntry::data(),
    );

    let tss_descriptor =
        TssDescriptor::new(
            core::ptr::addr_of!(TSS)
        );

    write_tss_descriptor(
        3,
        tss_descriptor,
    );

    let gdt_pointer = GdtPointer {
        limit:
            (core::mem::size_of::<Gdt>() - 1)
                as u16,
        base:
            core::ptr::addr_of!(GDT)
                as u64,
    };

    unsafe {
        load_gdt_and_segments(
            &gdt_pointer
        );

        load_tss();
    }
}