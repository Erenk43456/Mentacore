use super::framework::TestRunner;

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"cpu::user_gdt_segments",
        || crate::cpu::validate_user_segments(),
    );
}