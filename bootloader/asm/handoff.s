bits 64

section .text

global jump_to_kernel

jump_to_kernel:
    ; Disable interrupts before entering the kernel.
    cli

    ; Clear the direction flag for a clean kernel entry state.
    cld

    ; Windows x64 ABI:
    ; RCX = entry
    ; RDX = stack_top
    ; R8  = boot_info

    mov rax, rcx
    mov rsp, rdx

    ; Kernel uses the x86_64 SysV C ABI.
    ; A normal call would push an 8-byte return address,
    ; so emulate that stack alignment before entering via jmp.
    sub rsp, 8
    
    mov rdi, r8

    ; Kernel uses System V-style first argument:
    ; RDI = BootInfo
    ;
    ; Never returns.
    jmp rax