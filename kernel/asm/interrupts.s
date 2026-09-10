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

    ; Preserve the register-frame pointer before
    ; applying any ABI alignment padding.
    mov rdi, rsp

    ; SysV x86-64 ABI:
    ;   call-site RSP % 16 == 0
    test rsp, 8
    jz .divide_dispatch_aligned

    sub rsp, 8

.divide_dispatch_aligned:
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

    ; Preserve the register-frame pointer before
    ; applying any ABI alignment padding.
    mov rdi, rsp

    ; SysV x86-64 ABI:
    ;   call-site RSP % 16 == 0
    test rsp, 8
    jz .invalid_opcode_dispatch_aligned

    sub rsp, 8

.invalid_opcode_dispatch_aligned:
    call invalid_opcode_dispatch


; ------------------------------------------------------------
; Double Fault (#DF)
; ------------------------------------------------------------

double_fault_entry:
    cli

    ; Save the CPU RSP after the IST1 switch.
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

    ; Preserve the register-frame pointer.
    mov rdi, rsp

    ; CPU-pushed #DF error code.
    mov rsi, [rdi + 72]

    ; CPU RSP after the IST1 switch.
    mov rdx, [rdi + 64]

    ; Normalize the SysV x86-64 call-site alignment.
    test rsp, 8
    jz .double_fault_dispatch_aligned

    sub rsp, 8

.double_fault_dispatch_aligned:
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

    ; Preserve the register-frame pointer before
    ; applying ABI alignment padding.
    mov rdi, rsp

    ; CPU-pushed error code is at [register_frame + 72].
    mov rsi, [rdi + 72]

    ; SysV x86-64 ABI:
    ;   call-site RSP % 16 == 0
    test rsp, 8
    jz .general_protection_dispatch_aligned

    sub rsp, 8

.general_protection_dispatch_aligned:
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

    ; Preserve the register-frame pointer before
    ; applying ABI alignment padding.
    mov rdi, rsp

    ; CPU-pushed error code is at [register_frame + 72].
    mov rsi, [rdi + 72]

    ; Normalize the call-site stack alignment.
    test rsp, 8
    jz .page_fault_dispatch_aligned

    sub rsp, 8

    call page_fault_dispatch

    ; Padding was added, so remove it explicitly.
    add rsp, 8

    jmp .page_fault_restore

.page_fault_dispatch_aligned:
    call page_fault_dispatch

.page_fault_restore:
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rax

    ; Discard CPU-pushed page-fault error code.
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

    ; SysV x86-64 ABI:
    ;   call-site RSP % 16 == 0

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
