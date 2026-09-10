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
            options(
                nomem,
                nostack,
                preserves_flags
            )
        );
    }

    ((high as u64) << 32) | (low as u64)
}