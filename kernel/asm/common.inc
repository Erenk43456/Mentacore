bits 64

INTERRUPT_FRAME_WORD_SIZE equ 8

REGISTER_FRAME_SIZE       equ 9 * INTERRUPT_FRAME_WORD_SIZE
CPU_ERROR_CODE_OFFSET     equ REGISTER_FRAME_SIZE
CPU_RIP_OFFSET            equ CPU_ERROR_CODE_OFFSET + INTERRUPT_FRAME_WORD_SIZE
CPU_CS_OFFSET             equ CPU_RIP_OFFSET + INTERRUPT_FRAME_WORD_SIZE
CPU_RFLAGS_OFFSET         equ CPU_CS_OFFSET + INTERRUPT_FRAME_WORD_SIZE
DF_CPU_RSP_OFFSET         equ REGISTER_FRAME_SIZE - INTERRUPT_FRAME_WORD_SIZE

; ------------------------------------------------------------
; Interrupt register-frame layout
;
; The entry stubs push registers in this order:
;   rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11
;
; Resulting frame:
;
;   +00  r11
;   +08  r10
;   +16  r9
;   +24  r8
;   +32  rdi
;   +40  rsi
;   +48  rdx
;   +56  rcx
;   +64  rax
;   +72  CPU error code
;   +80  CPU RIP
;   +88  CPU CS
;   +96  CPU RFLAGS
;
; For #DF, +64 contains the CPU RSP saved before
; the register pushes by double_fault_entry.
; ------------------------------------------------------------