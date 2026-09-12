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

global interrupt_context_switch

; void interrupt_context_switch(
;     u64* current_rsp,
;     const InterruptContext* next
; )
;
; InterruptContext:
;   +0   r15
;   +8   r14
;   +16  r13
;   +24  r12
;   +32  rbp
;   +40  rbx
;   +48  r11
;   +56  r10
;   +64  r9
;   +72  r8
;   +80  rdi
;   +88  rsi
;   +96  rdx
;   +104 rcx
;   +112 rax
;   +120 rip
;   +128 cs
;   +136 rflags
;   +144 rsp
;   +152 ss

interrupt_context_switch:
    ; RDI = current_rsp
    ; RSI = next InterruptContext

    mov [rdi], rsp

    mov rsp, rsi

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
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