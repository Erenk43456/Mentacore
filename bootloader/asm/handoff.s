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
    ; R9  = pml4_address

    mov rax, rcx

    ; Kernel uses System V-style first argument:
    ; RDI = BootInfo
    mov rdi, r8

    ; Switch to the bootloader-controlled page table.
    mov cr3, r9

    ; Switch to the kernel stack.
    mov rsp, rdx

    ; A normal call would push an 8-byte return address.
    ; The kernel entry is reached with jmp, so emulate that
    ; stack layout for the SysV ABI.
    sub rsp, 8

    ; Never returns.
    jmp rax