bits 64

section .text

extern divide_error_dispatch
extern invalid_opcode_dispatch
extern double_fault_dispatch
extern general_protection_dispatch
extern page_fault_dispatch
extern timer_irq_dispatch
extern lapic_timer_dispatch

global divide_error_entry
global invalid_opcode_entry
global double_fault_entry
global general_protection_entry
global page_fault_entry
global timer_irq_entry
global lapic_timer_entry


; ------------------------------------------------------------
; Divide Error (#DE)
; ------------------------------------------------------------

divide_error_entry:
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

    mov rdi, rsp
    call divide_error_dispatch


; ------------------------------------------------------------
; Invalid Opcode (#UD)
; ------------------------------------------------------------

invalid_opcode_entry:
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

    mov rdi, rsp
    call invalid_opcode_dispatch


; ------------------------------------------------------------
; Double Fault (#DF)
; ------------------------------------------------------------

double_fault_entry:
    cli

    mov rax, rsp

    push rax
    push rcx
    push rdx
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11

    mov rdi, rsp

    ; CPU-pushed #DF error code.
    mov rsi, [rsp + 72]

    ; First pushed RAX = CPU RSP after IST switch.
    mov rdx, [rsp + 64]

    call double_fault_dispatch


; ------------------------------------------------------------
; General Protection Fault (#GP)
; ------------------------------------------------------------

general_protection_entry:
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

    mov rdi, rsp
    mov rsi, [rsp + 72]

    call general_protection_dispatch


; ------------------------------------------------------------
; Page Fault (#PF)
; ------------------------------------------------------------

page_fault_entry:
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

    mov rdi, rsp
    mov rsi, [rsp + 72]

    call page_fault_dispatch

    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax

    add rsp, 8

    iretq


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

    ; RSP at this point is the stack pointer
    ; immediately before the Rust call.
    ;
    ; SysV x86-64 ABI:
    ;   call-site RSP % 16 == 0
    ;   Rust entry RSP % 16 == 8

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

    call lapic_timer_dispatch

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
