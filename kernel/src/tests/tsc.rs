use crate::cpu;

use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"cpu::tsc_calibration",
        test_tsc_calibration,
    );
}

fn test_tsc_calibration() -> bool {
    let tsc_frequency = cpu::calibrate_tsc();

    tsc_frequency != 0
}