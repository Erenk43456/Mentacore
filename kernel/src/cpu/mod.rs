mod gdt;
mod tss;

mod features;
mod msr;
mod tsc;
mod state;

#[cfg(feature = "verbose-boot")]
pub use features::CpuInfo;

pub use state::InterruptState;

#[cfg(feature = "kernel-tests")]
pub use state::interrupts_enabled;

#[cfg(feature = "kernel-tests")]
pub use tsc::{calibrate_tsc, read_tsc};

pub use msr::read_msr;

pub use tss::ist1_stack_range;

#[cfg(feature = "verbose-boot")]
pub use tss::ist1_stack_top;

#[inline]
pub fn halt() {
    state::halt();
}

pub fn init() {
    let gdt_ptr =
        core::ptr::addr_of_mut!(gdt::GDT);

    unsafe {
        core::ptr::write_bytes(
            gdt_ptr as *mut u8,
            0,
            core::mem::size_of::<gdt::Gdt>(),
        );

        gdt::write_gdt_entry(
            gdt_ptr,
            1,
            gdt::GdtEntry::code(),
        );

        gdt::write_gdt_entry(
            gdt_ptr,
            2,
            gdt::GdtEntry::data(),
        );
    }

    tss::initialize_stacks();

    let tss_descriptor =
        gdt::TssDescriptor::new(
            core::ptr::addr_of!(tss::TSS),
        );

    unsafe {
        gdt::write_tss_descriptor(
            gdt_ptr,
            3,
            tss_descriptor,
        );

        gdt::write_gdt_entry(
            gdt_ptr,
            5,
            gdt::GdtEntry::user_code(),
        );

        gdt::write_gdt_entry(
            gdt_ptr,
            6,
            gdt::GdtEntry::user_data(),
        );

        gdt::load(
            core::ptr::addr_of!(gdt::GDT),
        );
    }
}

#[cfg(feature = "kernel-tests")]
pub fn validate_user_segments() -> bool {
    gdt::validate_user_segments()
}