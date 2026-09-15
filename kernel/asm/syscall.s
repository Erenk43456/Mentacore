bits 64

section .text

extern syscall_interrupt_dispatch

global syscall_interrupt_entry

syscall_interrupt_entry:
    cli

    ; Save all general-purpose registers.
    ;
    ; Stack after pushes:
    ;   +0   RBX
    ;   +8   RBP
    ;   +16  R15
    ;   +24  R14
    ;   +32  R13
    ;   +40  R12
    ;   +48  R11
    ;   +56  R10
    ;   +64  R9
    ;   +72  R8
    ;   +80  RDI
    ;   +88  RSI
    ;   +96  RDX
    ;   +104 RCX
    ;   +112 RAX
    ;
    ; CPU interrupt frame:
    ;   +120 RIP
    ;   +128 CS
    ;   +136 RFLAGS
    ;   +144 RSP
    ;   +152 SS

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
    call syscall_interrupt_dispatch

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