bits 64

section .text

global load_gdt_and_segments

load_gdt_and_segments:
    lgdt [rdi]

    ; KERNEL_CODE_SELECTOR = 0x08
    push 0x08

    lea rax, [rel .reload_cs]
    push rax

    retfq

.reload_cs:

    ; KERNEL_DATA_SELECTOR = 0x10
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ret

global context_switch

; void context_switch(
;     KernelContext* current,
;     const KernelContext* next
; )
;
; KernelContext layout:
;   +0   rsp
;   +8   rip
;   +16  rflags
;   +24  rbx
;   +32  rbp
;   +40  r12
;   +48  r13
;   +56  r14
;   +64  r15

context_switch:
    ; RDI = current context
    ; RSI = next context

    ; Save callee-saved registers.
    mov [rdi + 24], rbx
    mov [rdi + 32], rbp
    mov [rdi + 40], r12
    mov [rdi + 48], r13
    mov [rdi + 56], r14
    mov [rdi + 64], r15

    ; Save the post-return stack pointer.
    ;
    ; The return address at [rsp] belongs to this
    ; context_switch call. When this context is
    ; restored, execution jumps directly to that
    ; return address, so RSP must point past it.
    lea rax, [rsp + 8]
    mov [rdi + 0], rax

    ; Save the return address as the instruction pointer.
    mov rax, [rsp]
    mov [rdi + 8], rax

    ; Save current RFLAGS.
    pushfq
    pop rax
    mov [rdi + 16], rax

    ; Load next context.
    mov rsp, [rsi + 0]
    mov rbx, [rsi + 24]
    mov rbp, [rsi + 32]
    mov r12, [rsi + 40]
    mov r13, [rsi + 48]
    mov r14, [rsi + 56]
    mov r15, [rsi + 64]

    ; Restore RFLAGS.
    mov rax, [rsi + 16]
    push rax
    popfq

    ; Continue execution at next context.
    mov rax, [rsi + 8]
    jmp rax

extern context_switch_test_target

global context_switch_test_trampoline

context_switch_test_trampoline:
    call context_switch_test_target

    ; The test target must never return.

    hlt