pub(super) fn test_double_fault_ist1() -> bool {
    crate::interrupts::trigger_double_fault_test()
}