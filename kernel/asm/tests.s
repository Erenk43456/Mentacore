%include "common.inc"

section .text

extern context_switch_test_target
extern ring3_test_syscall_interrupt_dispatch

global context_switch_test_trampoline

context_switch_test_trampoline:
    call context_switch_test_target

    ; The test target must never return.
    hlt


; ------------------------------------------------------------
; Double-fault recovery test entry
; ------------------------------------------------------------

extern DOUBLE_FAULT_TEST_RESUME_RIP
extern DOUBLE_FAULT_TEST_RESUME_RSP

global double_fault_test_entry

double_fault_test_entry:
    ; Save the stack pointer of the original kernel test call.
    ;
    ; After #DF, the CPU switches to IST1. The original RSP
    ; is therefore not automatically restored by IRETQ.
    mov [rel DOUBLE_FAULT_TEST_RESUME_RSP], rsp

    ; Store an explicit continuation address for the test handler.
    lea rax, [rel .resume]
    mov [rel DOUBLE_FAULT_TEST_RESUME_RIP], rax

    ; The page-fault IDT entry was deliberately cleared by the
    ; Rust test before entering this function.
    ;
    ; Accessing the unmapped address therefore produces:
    ;
    ;   #PF -> no #PF handler -> #DF
    ;
    mov rdi, 0x0000_5000_0000_0000
    mov al, [rdi]

    ; Reaching this point means the expected double fault did not
    ; occur.
    xor eax, eax
    ret

.resume:
    ; IRETQ returned us from the #DF IST1 stack.
    ; Restore the original kernel test stack before RET.
    mov rsp, [rel DOUBLE_FAULT_TEST_RESUME_RSP]

    mov eax, 1

    ; The test intentionally executed CLI before triggering #DF.
    ; Re-enable interrupts before returning to the test runner.
    sti

    ret


; ------------------------------------------------------------
; Ring 3 privilege-transition test entry
; ------------------------------------------------------------

section .user_text progbits alloc exec nowrite align=4096

global user_privilege_test

user_privilege_test:
    ; SYS_GET_TID
    mov rax, 0
    int 0x80

    ; Current thread ID must be a valid small ThreadId.
    cmp rax, 63
    ja .failure

    ; Tell the kernel-side test that the syscall succeeded.
    mov rdi, 1
    mov rax, -1
    int 0x80
    hlt

.failure:
    xor rdi, rdi
    mov rax, -1
    int 0x80
    hlt


; ------------------------------------------------------------
; Ring 3 test interrupt entry
; ------------------------------------------------------------

section .text

global user_privilege_interrupt_entry

user_privilege_interrupt_entry:
    cli

    ; Save all general-purpose registers.
    ;
    ; 15 registers * 8 bytes = 120 bytes.
    ;
    ; After these pushes:
    ;   RSP + 120 = CPU-created interrupt frame
    ;
    ; CPU frame:
    ;   +0  RIP
    ;   +8  CS
    ;   +16 RFLAGS
    ;   +24 RSP
    ;   +32 SS

    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    push rbp
    push rbx

    mov rdi, rsp
    lea rsi, [rsp + 120]
    mov rdx, rsp

    ; SysV ABI:
    ; RDI = saved register frame
    ; RSI = CPU interrupt frame
    ; RDX = current kernel RSP

    test rsp, 8
    jz .dispatch_aligned

    sub rsp, 8

.dispatch_aligned:
    call ring3_test_syscall_interrupt_dispatch

    test rsp, 8
    jz .restore_aligned

    add rsp, 8

.restore_aligned:
    pop rbx
    pop rbp
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax

    iretq


; ------------------------------------------------------------
; Return from Ring 3 test back into the Rust test function.
;
; This is reached through an IRETQ whose target CS is CPL0.
;
; Same-CPL IRETQ consumes:
;   RIP / CS / RFLAGS
;
; leaving the old user RSP/SS words on the kernel stack.
;
; Skip those two words and RET through the original
; enter_user_mode() call frame.
; ------------------------------------------------------------

global ring3_kernel_resume

ring3_kernel_resume:
    ret


; ------------------------------------------------------------
; Enter Ring 3 from the kernel.
;
; RDI = user RIP
; RSI = user RSP
; ------------------------------------------------------------

extern RING3_KERNEL_RESUME_RSP

global enter_user_mode
enter_user_mode:
    mov [rel RING3_KERNEL_RESUME_RSP], rsp

    push 0x33
    push rsi
    push 0x202
    push 0x2B
    push rdi
    iretq