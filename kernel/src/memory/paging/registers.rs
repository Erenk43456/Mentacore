use super::{
    PageTable,
    ADDRESS_MASK,
};

pub unsafe fn current_pml4() -> *mut PageTable {
    let address: u64;

    unsafe {
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) address,
            options(nostack, preserves_flags)
        );
    }

    (address & ADDRESS_MASK) as *mut PageTable
}

pub(super) unsafe fn load_cr3(
    address: u64,
) {
    unsafe {
        core::arch::asm!(
            "mov cr3, {0}",
            in(reg) address,
            options(nostack, preserves_flags)
        );
    }
}