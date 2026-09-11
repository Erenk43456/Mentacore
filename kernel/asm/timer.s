bits 64

%include "common.inc"

section .text

extern timer_irq_dispatch
extern lapic_timer_dispatch

global timer_irq_entry
global lapic_timer_entry


; ------------------------------------------------------------
; Legacy PIT Timer IRQ
; ------------------------------------------------------------

timer_irq_entry:
    cli

    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    mov rdi, rsp

    test rsp, 8
    jz .timer_dispatch_aligned

    sub rsp, 8

    call timer_irq_dispatch

    add rsp, 8
    jmp .timer_dispatch_done

.timer_dispatch_aligned:
    call timer_irq_dispatch

.timer_dispatch_done:
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


; ------------------------------------------------------------
; LAPIC Timer IRQ
; ------------------------------------------------------------

lapic_timer_entry:
    cli

    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push rbx
    push rbp
    push r12
    push r13
    push r14
    push r15

    mov rdi, rsp

    test rsp, 8
    jz .lapic_timer_dispatch_aligned

    sub rsp, 8

    call lapic_timer_dispatch

    add rsp, 8
    jmp .lapic_timer_dispatch_done

.lapic_timer_dispatch_aligned:
    call lapic_timer_dispatch

.lapic_timer_dispatch_done:
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