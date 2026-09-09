bits 64

section .text

global jump_to_kernel

jump_to_kernel:
    ; Windows x64 ABI:
    ; RCX = entry
    ; RDX = stack_top
    ; R8  = boot_info

    mov rax, rcx
    mov rsp, rdx
    mov rdi, r8

    ; Kernel uses System V-style first argument:
    ; RDI = BootInfo
    ;
    ; Never returns.
    jmp rax