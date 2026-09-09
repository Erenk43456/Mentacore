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

#[cfg(feature = "verbose-boot")]
#[derive(Clone, Copy)]
pub struct CpuInfo {
    pub vendor: [u8; 12],
    pub max_basic_leaf: u32,

    pub has_apic: bool,
    pub has_x2apic: bool,
    pub has_tsc: bool,
    pub has_msr: bool,
    pub has_sse: bool,
    pub has_sse2: bool,
    pub has_xsave: bool,
}

#[cfg(feature = "verbose-boot")]
impl CpuInfo {
    pub fn detect() -> Self {
        let (max_basic_leaf, ebx, ecx, edx) =
            cpuid(0);

        let mut vendor = [0u8; 12];

        vendor[0..4]
            .copy_from_slice(&ebx.to_le_bytes());

        vendor[4..8]
            .copy_from_slice(&edx.to_le_bytes());

        vendor[8..12]
            .copy_from_slice(&ecx.to_le_bytes());

        let (feature_ecx, feature_edx) =
            if max_basic_leaf >= 1 {
                let (_, _, ecx, edx) =
                    cpuid(1);

                (ecx, edx)
            } else {
                (0, 0)
            };

        let has_apic =
            (feature_edx & (1 << 9)) != 0;

        let has_tsc =
            (feature_edx & (1 << 4)) != 0;

        let has_msr =
            (feature_edx & (1 << 5)) != 0;

        let has_sse =
            (feature_edx & (1 << 25)) != 0;

        let has_sse2 =
            (feature_edx & (1 << 26)) != 0;

        let has_xsave =
            (feature_ecx & (1 << 26)) != 0;

        let has_x2apic =
            (feature_ecx & (1 << 21)) != 0;

        Self {
            vendor,
            max_basic_leaf,

            has_apic,
            has_x2apic,
            has_tsc,
            has_msr,
            has_sse,
            has_sse2,
            has_xsave,
        }
    }
}

#[cfg(feature = "verbose-boot")]
#[inline]
fn cpuid(leaf: u32) -> (u32, u32, u32, u32) {
    let result = core::arch::x86_64::__cpuid(leaf);

    (
        result.eax,
        result.ebx,
        result.ecx,
        result.edx,
    )
}

#[inline]
pub fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;

    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }

    ((high as u64) << 32) | (low as u64)
}

#[cfg(feature = "kernel-tests")]
#[inline]
pub fn read_tsc() -> u64 {
    unsafe {
        let low: u32;
        let high: u32;

        core::arch::asm!(
            "lfence",
            "rdtsc",
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );

        ((high as u64) << 32) | (low as u64)
    }
}

#[cfg(feature = "kernel-tests")]
pub fn calibrate_tsc() -> u64 {
    const CALIBRATION_TICKS: u64 = 100;

    let start_ticks =
        crate::interrupts::timer_ticks();

    let target_ticks =
        start_ticks + CALIBRATION_TICKS;

    let start_tsc = read_tsc();

    while crate::interrupts::timer_ticks()
        < target_ticks
    {
        core::hint::spin_loop();
    }

    let end_tsc = read_tsc();

    let elapsed_tsc =
        end_tsc - start_tsc;

    let pit_frequency =
        crate::hardware::pit::actual_frequency(100);

    if pit_frequency == 0 {
        return 0;
    }

    let frequency =
        ((elapsed_tsc as u128)
            * (pit_frequency as u128)
            / (CALIBRATION_TICKS as u128))
            as u64;

    frequency
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

            iomap_base:
                core::mem::size_of::<Self>() as u16,
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
            granularity:
                ((limit >> 16) & 0x0F) as u8,
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

#[repr(align(16))]
struct Stack<const SIZE: usize> {
    data: [u8; SIZE],
}

const KERNEL_STACK_SIZE: usize =
    16 * 1024;

const IST1_STACK_SIZE: usize =
    16 * 1024;

static mut KERNEL_STACK:
    Stack<KERNEL_STACK_SIZE> =
    Stack {
        data: [0; KERNEL_STACK_SIZE],
    };

static mut IST1_STACK:
    Stack<IST1_STACK_SIZE> =
    Stack {
        data: [0; IST1_STACK_SIZE],
    };

static mut TSS: Tss = Tss::new();

static mut GDT: Gdt = Gdt {
    entries: [0; 5],
};

fn write_gdt_entry(
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
        GDT.entries[index] = value;
    }
}

fn write_tss_descriptor(
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
        GDT.entries[index] = low;

        GDT.entries[index + 1] =
            (descriptor.base_upper as u64)
            | ((descriptor.reserved as u64) << 32);
    }
}

fn initialize_tss_stacks() {
    let (kernel_stack_top, ist1_stack_top) = unsafe {
        (
            core::ptr::addr_of!(KERNEL_STACK.data) as u64
                + KERNEL_STACK_SIZE as u64,

            core::ptr::addr_of!(IST1_STACK.data) as u64
                + IST1_STACK_SIZE as u64,
        )
    };

    unsafe {
        let tss_ptr =
            core::ptr::addr_of_mut!(TSS)
                as *mut u8;

        core::ptr::write_unaligned(
            tss_ptr.add(4) as *mut u64,
            kernel_stack_top,
        );

        core::ptr::write_unaligned(
            tss_ptr.add(36) as *mut u64,
            ist1_stack_top,
        );
    }
}

pub fn ist1_stack_top() -> u64 {
    core::ptr::addr_of!(IST1_STACK) as u64
        + IST1_STACK_SIZE as u64
}

#[cfg(feature = "verbose-boot")]
pub fn tss_ist1() -> u64 {
    unsafe {
        core::ptr::read_unaligned(
            (core::ptr::addr_of!(TSS)
                as *const u8)
                .add(36)
                as *const u64,
        )
    }
}

unsafe extern "C" {
    fn load_gdt_and_segments(
        gdt_pointer: *const GdtPointer,
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

#[derive(Clone, Copy)]
pub struct InterruptState {
    rflags: u64,
}

impl InterruptState {
    #[inline]
    pub fn save_and_disable() -> Self {
        let rflags: u64;

        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {}",
                out(reg) rflags,
                options(nomem, preserves_flags)
            );

            core::arch::asm!(
                "cli",
                options(nostack)
            );
        }

        Self { rflags }
    }

    #[inline]
    pub fn restore(self) {
        if self.rflags & (1 << 9) != 0 {
            unsafe {
                core::arch::asm!(
                    "sti",
                    options(nostack)
                );
            }
        }
    }

    #[cfg(feature = "kernel-tests")]
    #[inline]
    pub fn were_enabled(self) -> bool {
        self.rflags & (1 << 9) != 0
    }
}

#[cfg(feature = "kernel-tests")]
#[inline]
pub fn interrupts_enabled() -> bool {
    let rflags: u64;

    unsafe {
        core::arch::asm!(
            "pushfq",
            "pop {}",
            out(reg) rflags,
            options(nomem, preserves_flags)
        );
    }

    rflags & (1 << 9) != 0
}

#[inline]
pub fn halt() {
    unsafe {
        core::arch::asm!(
            "hlt",
            options(
                nomem,
                nostack,
                preserves_flags
            )
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

    initialize_tss_stacks();

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