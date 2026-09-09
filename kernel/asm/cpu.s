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
