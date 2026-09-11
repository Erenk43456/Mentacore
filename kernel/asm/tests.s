bits 64

section .text

extern context_switch_test_target

global context_switch_test_trampoline

context_switch_test_trampoline:
    call context_switch_test_target

    ; The test target must never return.

    hlt