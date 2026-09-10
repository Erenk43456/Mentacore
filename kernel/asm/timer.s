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
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11

    test rsp, 8
    jz .timer_dispatch_aligned

    sub rsp, 8

    mov rdi, rsp
    call timer_irq_dispatch

    add rsp, 8
    jmp .timer_dispatch_done

.timer_dispatch_aligned:
    mov rdi, rsp
    call timer_irq_dispatch

.timer_dispatch_done:
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
    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11

    test rsp, 8
    jz .lapic_timer_dispatch_aligned

    sub rsp, 8

    mov rdi, rsp
    call lapic_timer_dispatch

    add rsp, 8
    jmp .lapic_timer_dispatch_done

.lapic_timer_dispatch_aligned:
    mov rdi, rsp
    call lapic_timer_dispatch

.lapic_timer_dispatch_done:
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