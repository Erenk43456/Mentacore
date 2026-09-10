bits 64

%include "common.inc"

section .text

extern divide_error_dispatch
extern invalid_opcode_dispatch
extern double_fault_dispatch
extern general_protection_dispatch
extern page_fault_dispatch

global divide_error_entry
global invalid_opcode_entry
global double_fault_entry
global general_protection_entry
global page_fault_entry


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
    push 0

    mov rdi, rsp

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
    push 0

    mov rdi, rsp

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

    mov rsi, [rdi + CPU_ERROR_CODE_OFFSET]
    mov rdx, [rdi + DF_CPU_RSP_OFFSET]

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

    mov rdi, rsp

    mov rsi, [rdi + CPU_ERROR_CODE_OFFSET]

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

    mov rdi, rsp

    mov rsi, [rdi + CPU_ERROR_CODE_OFFSET]

    test rsp, 8
    jz .page_fault_dispatch_aligned

    sub rsp, 8

    call page_fault_dispatch

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

    add rsp, 8

    iretq