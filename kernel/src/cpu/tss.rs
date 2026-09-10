#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(super) struct Tss {
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
    pub(super) const fn new() -> Self {
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

#[repr(align(16))]
pub(super) struct Stack<const SIZE: usize> {
    pub(super) data: [u8; SIZE],
}

pub(super) const KERNEL_STACK_SIZE: usize = 16 * 1024;
pub(super) const IST1_STACK_SIZE: usize = 16 * 1024;

pub(super) static mut KERNEL_STACK: Stack<KERNEL_STACK_SIZE> =
    Stack {
        data: [0; KERNEL_STACK_SIZE],
    };

pub(super) static mut IST1_STACK: Stack<IST1_STACK_SIZE> =
    Stack {
        data: [0; IST1_STACK_SIZE],
    };

pub(super) static mut TSS: Tss = Tss::new();

pub(super) fn initialize_stacks() {
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
            core::ptr::addr_of_mut!(TSS) as *mut u8;

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

pub fn ist1_stack_range() -> (u64, u64) {
    let start =
        core::ptr::addr_of!(IST1_STACK) as u64;

    let end =
        start + IST1_STACK_SIZE as u64;

    (start, end)
}

#[cfg(feature = "verbose-boot")]
pub(super) fn ist1_stack_top() -> u64 {
    unsafe {
        core::ptr::read_unaligned(
            (core::ptr::addr_of!(TSS)
                as *const u8)
                .add(36)
                as *const u64,
        )
    }
}