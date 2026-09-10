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
            options(
                nomem,
                nostack,
                preserves_flags
            )
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

    let start_tsc =
        read_tsc();

    while crate::interrupts::timer_ticks()
        < target_ticks
    {
        core::hint::spin_loop();
    }

    let end_tsc =
        read_tsc();

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