use super::{
    PageTable,
    ADDRESS_MASK,
};

pub unsafe fn current_pml4_address() -> u64 {
    let address: u64;

    unsafe {
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) address,
            options(nostack, preserves_flags)
        );
    }

    address & ADDRESS_MASK
}

pub unsafe fn current_pml4() -> *mut PageTable {
    match unsafe {
        super::table::physical_table_pointer(
            current_pml4_address(),
        )
    } {
        Ok(ptr) => ptr,

        Err(()) => {
            crate::debug::write(
                b"FATAL: CR3 contains invalid physical address\r\n",
            );

            loop {
                core::hint::spin_loop();
            }
        }
    }
}

pub unsafe fn load_cr3(
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