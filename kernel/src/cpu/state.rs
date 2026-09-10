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
                options(
                    nomem,
                    preserves_flags
                )
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
            options(
                nomem,
                preserves_flags
            )
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