# TSS / Kernel Stack ABI

## Purpose

The TSS / Kernel Stack ABI defines the CPU stack-selection contract used when execution enters the kernel from a different privilege level or through an IDT entry configured with an Interrupt Stack Table (IST) entry.

It establishes the kernel stack used for CPL3 → CPL0 transitions and the dedicated stack used by double-fault handling.

## Contract-Defining Files

### `kernel/src/cpu/tss.rs`

Defines the 64-bit TSS representation, kernel stack, IST1 stack, and the values installed into the corresponding TSS fields.

The relevant architectural fields are:

```text id="2yq0c9"
RSP0  → kernel stack top
IST1  → dedicated double-fault stack top
```

The current stack sizes are:

```text id="6w6x8d"
KERNEL_STACK_SIZE = 16 KiB
IST1_STACK_SIZE   = 16 KiB
```

`initialize_stacks()` writes the stack tops into the TSS before the TSS is loaded.

### `kernel/src/cpu/gdt.rs`

Defines the TSS descriptor and its selector:

```text id="r3m8hv"
TSS_SELECTOR = 0x18
```

The descriptor occupies two consecutive GDT entries because a 64-bit TSS descriptor spans 16 bytes.

The GDT is loaded and the TSS selector is subsequently loaded with `ltr`.

### `kernel/src/interrupts/idt.rs`

Defines which IDT entries use the TSS-based IST mechanism.

The double-fault entry is configured with:

```text id="e9f4cy"
IST = 1
```

Other kernel exception and interrupt entries currently use `IST = 0`.

## Kernel Stack Contract

The TSS `RSP0` field contains the top of the kernel's dedicated entry stack.

When the processor transitions from CPL3 to CPL0 through an applicable interrupt/trap gate, the CPU uses this stack as the kernel entry stack.

The contract is therefore:

```text id="b6t8yy"
TSS.RSP0
    │
    ▼
kernel stack top
    │
    ▼
kernel entry frame
```

The stack must be valid and mapped before userspace execution can safely transition into the kernel.

## IST1 Contract

The TSS `IST1` field contains the top of a dedicated stack reserved for handlers configured with IST index 1.

The double-fault IDT entry uses IST1:

```text id="6zzw9x"
IDT[8]
    │
    └── IST = 1
             │
             ▼
         TSS.IST1
             │
             ▼
       IST1 stack top
```

This provides a separate stack for double-fault entry instead of relying on the currently active stack.

## Stack Representation

The stacks are statically allocated as aligned byte arrays:

```text id="1ygr6a"
#[repr(align(16))]
struct Stack<const SIZE: usize> {
    data: [u8; SIZE],
}
```

The ABI uses the **top address** of each stack, not its base address.

The stack grows downward from that top address.

## TSS Representation

The TSS structure is `#[repr(C, packed)]`.

The relevant 64-bit fields are represented as adjacent low/high 32-bit values:

```text id="0f6j6r"
RSP0 = rsp0_low  + rsp0_high
IST1 = ist1_low  + ist1_high
```

The implementation writes the complete 64-bit stack addresses into these locations before loading the TSS.

## Critical Low-Level Convention

### TSS loaded before privilege transitions

The GDT/TSS initialization sequence establishes the TSS and executes `ltr` before the kernel relies on the CPU to perform privilege-level stack switching.

Consequently, the TSS stack contract is a prerequisite for the Userspace Entry ABI and syscall/exception entry paths.

### IST selection is an IDT property

The CPU selects IST1 because the corresponding IDT entry contains an IST index of `1`.

The double-fault handler therefore depends on both:

```text id="b7j5u8"
TSS.IST1
IDT[8].IST = 1
```

Changing one without the other breaks the stack-selection contract.

## Boundary Summary

```text
TSS initialization
    │
    ├── RSP0 ──► kernel entry stack
    │
    └── IST1 ──► double-fault stack
    │
    ▼
GDT / LTR
    │
    ▼
CPU exception / interrupt entry
    │
    ├── CPL transition ──► RSP0
    │
    └── IST=1 ───────────► IST1
```

The **TSS / Kernel Stack ABI** defines CPU-managed stack selection at entry boundaries. It does not define per-thread kernel stacks or the interrupt frame layout restored after scheduling.
