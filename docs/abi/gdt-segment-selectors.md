# GDT / Segment Selector ABI

## Purpose

The GDT / Segment Selector ABI defines the segment descriptors and selector values relied upon by the kernel's privilege-transition and interrupt-return mechanisms.

The contract is shared by GDT initialization, exception/interrupt return frames, userspace startup, and TSS loading.

## Contract-Defining Files

### `kernel/src/cpu/gdt.rs`

Defines the GDT representation, descriptor contents, selector values, and GDT loading procedure.

The externally significant selector constants are:

```text id="1q6q8v"
KERNEL_CODE_SELECTOR = 0x08
KERNEL_DATA_SELECTOR = 0x10
TSS_SELECTOR          = 0x18

USER_CODE_SELECTOR    = 0x2B
USER_DATA_SELECTOR    = 0x33
```

It also defines the descriptor types used to construct the GDT.

### `kernel/src/cpu/mod.rs`

Defines the kernel-visible selector boundary by re-exporting the selectors required by other CPU and thread subsystems.

The implementation of the descriptors remains private to the GDT module.

### `kernel/src/thread/interrupt_context.rs`

Consumes the user and kernel code/data selectors when constructing interrupt-return contexts.

The selectors placed into `CS` and `SS` must correspond to the descriptors established by the GDT ABI.

### `kernel/src/thread/thread.rs`

Consumes the kernel code selector when constructing the initial kernel interrupt context.

## GDT Layout

The current GDT entry arrangement is:

```text id="o7g3v1"
Index  Selector  Descriptor
-----  --------  -----------------------
0      0x00      Null descriptor
1      0x08      Kernel code
2      0x10      Kernel data
3      0x18      TSS
4      0x20      TSS continuation
5      0x28      User code
6      0x30      User data
```

The user selectors contain the requested privilege level in their low bits:

```text id="7mlj1h"
0x28 | 3 = 0x2B    user code
0x30 | 3 = 0x33    user data
```

The selector values are therefore coupled to both the GDT entry indices and the privilege level.

## Descriptor Contract

The descriptor types are:

```text id="6shw8w"
Kernel code  → executable, kernel privilege
Kernel data  → writable data, kernel privilege
User code    → executable, user privilege
User data    → writable data, user privilege
TSS          → available 64-bit TSS descriptor
```

The current descriptor access bytes are:

```text id="0i8j7u"
Kernel code  = 0x9A
Kernel data  = 0x92
User code    = 0xFA
User data    = 0xF2
TSS          = 0x89
```

These values are part of the descriptor contract because the selector values alone are meaningful only when the corresponding GDT entries have the expected type and privilege.

## Userspace Return Contract

A userspace `InterruptContext` must contain:

```text id="h4y7ib"
CS = USER_CODE_SELECTOR
SS = USER_DATA_SELECTOR
```

These selectors identify the CPL3 code and data descriptors consumed by `iretq`.

Consequently, changing either the selector value or the corresponding GDT entry changes the userspace entry ABI.

## Kernel Return Contract

Kernel interrupt contexts use:

```text id="gq2n2o"
CS = KERNEL_CODE_SELECTOR
```

This identifies the CPL0 code segment used when `iretq` returns to a kernel thread.

## TSS Contract

The TSS occupies two consecutive GDT entries because the 64-bit TSS descriptor is larger than a normal 8-byte descriptor:

```text id="0e1x9n"
TSS selector = 0x18
TSS descriptor = entries 3 and 4
```

`ltr` loads the TSS selector after the GDT has been installed.

The TSS selector therefore depends on the fixed GDT index assigned to the TSS descriptor.

## Critical Low-Level Conventions

### Selector values encode descriptor indices

The selector constants are not arbitrary numeric identifiers.

Their values depend on the GDT entry positions and requested privilege level. A change to the GDT layout can therefore invalidate every ABI consumer that embeds these selectors in CPU return frames.

### User privilege transition

The userspace transition relies on the combination:

```text id="h3p1w0"
USER_CODE_SELECTOR
USER_DATA_SELECTOR
RIP
RSP
RFLAGS
```

being restored together through `iretq`.

The GDT ABI provides the segment descriptors; the ELF Userspace Entry ABI provides the initial execution state that references them.

## Boundary Summary

```text id="tdk4p4"
GDT initialization
    │
    ├── kernel code/data descriptors
    ├── TSS descriptor
    └── user code/data descriptors
    │
    ▼
Selector constants
    │
    ├── kernel CS
    ├── user CS
    └── user SS
    │
    ▼
InterruptContext / KernelInterruptContext
    │
    ▼
iretq
    │
    ├── CPL0 kernel execution
    └── CPL3 userspace execution
```

The **GDT / Segment Selector ABI** defines the descriptor and selector contract. It does not define interrupt-frame layout, syscall conventions, or the ELF entry address itself.
