# IDT Gate / Interrupt Vector ABI

## Purpose

The IDT Gate / Interrupt Vector ABI defines the binary contract between the CPU's Interrupt Descriptor Table and Mentacore's interrupt entry stubs.

It establishes the layout of an IDT gate, the handler address encoding, privilege-level access, IST selection, and the interrupt-vector assignments used by the kernel.

## Contract-Defining Files

### `kernel/src/interrupts/idt.rs`

Defines the 64-bit IDT gate representation:

```text id="7q4m2s"
+00  offset_low    u16
+02  selector      u16
+04  options       u16
+06  offset_mid    u16
+08  offset_high   u32
+12  reserved      u32
```

Each entry is therefore 16 bytes, and the IDT contains 256 entries.

The handler address is reconstructed from:

```text id="2n8v5k"
offset_low
offset_mid << 16
offset_high << 32
```

The gate options encode:

```text id="9c3x7p"
Present
Interrupt Gate
DPL
IST index
```

Kernel handlers use an interrupt gate with DPL 0. The syscall gate is configured with DPL 3.

The file also defines the vector-to-entry-point mapping.

### `kernel/asm/common.inc`

Defines the shared interrupt-frame constants used by the assembly entry stubs.

The register frame consists of nine saved registers:

```text id="4m6q1v"
+00 r11
+08 r10
+16 r9
+24 r8
+32 rdi
+40 rsi
+48 rdx
+56 rcx
+64 rax
```

CPU-pushed exception state follows this register frame:

```text id="8k2r6y"
+72 error code
+80 RIP
+88 CS
+96 RFLAGS
```

The exception and interrupt entry ABIs build on this common CPU/assembly layout.

### `kernel/asm/exceptions.s`

Defines the exception entry symbols referenced by the IDT:

```text id="5v9p3c"
divide_error_entry
invalid_opcode_entry
double_fault_entry
general_protection_entry
page_fault_entry
```

### `kernel/asm/timer.s`

Defines the hardware timer entry symbols:

```text id="1x7m4q"
timer_irq_entry
lapic_timer_entry
```

### `kernel/asm/syscall.s`

Defines the user-accessible syscall entry symbol:

```text id="6f2n8w"
syscall_interrupt_entry
```

## Vector Contract

The current architectural vector assignments are:

```text id="3p7k5m"
Vector     Entry
------     -------------------------------
0          divide_error_entry
6          invalid_opcode_entry
8          double_fault_entry
13         general_protection_entry
14         page_fault_entry
32         timer_irq_entry
0x40       lapic_timer_entry
0x80       syscall_interrupt_entry
```

The double-fault vector additionally specifies:

```text id="2w9c6n"
IST = 1
```

All other listed kernel handlers currently use IST 0.

The syscall vector is configured with DPL 3 so that CPL3 software can invoke it.

## Gate Option Contract

Kernel interrupt gates are constructed from:

```text id="5h8q2r"
0x8E00 | IST
```

This represents:

```text
Present = 1
Gate type = 0xE (64-bit interrupt gate)
DPL = 0
```

The syscall/user-test gate uses:

```text id="7c4m9x"
0x8E00 | (3 << 13)
```

which gives the gate DPL 3.

## IDT Pointer Contract

The IDTR operand is represented as:

```text id="0m6v8p"
#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}
```

The limit is:

```text id="4r1n7k"
size_of::<IdtEntry>() * 256 - 1
```

The base is the address of the 256-entry IDT.

The resulting structure is passed directly to the CPU's `lidt` instruction.

## Critical Low-Level Convention

### Handler addresses are split across three fields

The CPU does not receive the handler address as a single Rust pointer. The 64-bit address is encoded into:

```text id="8s5q2m"
offset_low
offset_mid
offset_high
```

The three fields must reconstruct the exact assembly entry address.

### IST is part of the gate

The IST value is not selected by the assembly entry stub. It is selected by the CPU from the IDT gate before the entry stub executes.

Therefore:

```text id="3x9v6k"
IDT gate
   │
   └── IST = 1
          │
          ▼
       TSS.IST1
```

for the double-fault path.

## Boundary Summary

```text id="6p2m8r"
CPU interrupt/exception vector
          │
          ▼
       IDT[vector]
          │
          ├── handler address
          ├── CS selector
          ├── DPL
          └── IST
                │
                ▼
        assembly entry stub
                │
                ▼
        Rust dispatch function
```

The **IDT Gate / Interrupt Vector ABI** defines the CPU-visible descriptor and vector boundary. It does not define the register-frame semantics of each individual entry path; those are documented by the Exception Entry, Timer Interrupt, and Syscall ABIs.
