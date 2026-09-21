# Page Table / CR3 Address-Space Switch ABI

## Purpose

The Page Table / CR3 Address-Space Switch ABI defines the low-level contract between the kernel's address-space representation and the CPU's `CR3` register.

It establishes which value identifies an address space, how that value is loaded into `CR3`, and how an interrupt-context switch changes address spaces without accessing the target stack through the old address space.

## Contract-Defining Files

### `kernel/src/memory/paging/address_space.rs`

Defines the kernel representation of an address space.

Each `AddressSpace` stores:

```text id="g2s4aj"
pml4
pml4_address
```

`pml4_address` is the physical address of the PML4 frame and is the value used when activating the address space.

`AddressSpace::activate()` passes this physical address to `load_cr3()`.

### `kernel/src/memory/paging/registers.rs`

Defines the architectural `CR3` interface.

The module provides:

```text id="j1g5v7"
current_pml4()
current_pml4_address()
load_cr3()
```

The contract is that the active PML4 is identified by the physical address stored in `CR3`, with the architectural address bits extracted using the page-table address mask.

### `kernel/src/memory/paging/mod.rs`

Defines the page-table address representation shared by the paging subsystem.

The relevant constants are:

```text id="t9k1c4"
PAGE_SIZE
ADDRESS_MASK
PHYS_MAP_BASE
```

It also defines `PageTable` as the 512-entry PML4/page-table representation and exposes the address-space interface.

### `kernel/src/thread/context.rs`

Declares the assembly address-space switching interface:

```rust id="2g1k1a"
unsafe extern "C" {
    pub fn interrupt_context_switch_to_address_space(
        current_rsp: *mut u64,
        next: *const u64,
        next_pml4: u64,
    ) -> !;
}
```

The third argument is the physical PML4 address that must become the new `CR3` value.

### `kernel/asm/context_switch.s`

Defines the assembly-side ordering of the address-space switch.

The contract is:

```text id="k0p8v2"
RDI = current kernel-stack save location
RSI = next interrupt context
RDX = next PML4 physical address
```

The assembly performs the critical sequence:

```asm id="j2x9g6"
mov [rdi], rsp
mov cr3, rdx
mov rsp, rsi
```

The current stack is saved before changing address spaces. The target context stack is selected only after `CR3` has been loaded.

## CR3 Contract

`CR3` contains the physical address of the active PML4.

The contract is therefore:

```text id="y4s5nf"
AddressSpace
    │
    └── pml4_address
            │
            ▼
          CR3
            │
            ▼
       active PML4
```

The PML4 address must be page-aligned and represent the physical frame containing the root page table.

## Address Representation

The paging subsystem uses:

```text id="y4l1qh"
ADDRESS_MASK = 0x000f_ffff_ffff_f000
```

to extract the physical page-table address from a page-table entry or `CR3`.

The kernel also maintains a physical-memory mapping beginning at:

```text id="9x0q6z"
PHYS_MAP_BASE = 0xFFFF_9000_0000_0000
```

This mapping allows physical page-table frames to be accessed through their corresponding kernel virtual addresses when required by the paging implementation.

## Address-Space Switch Ordering

The ordering in `interrupt_context_switch_to_address_space` is part of the ABI.

The required sequence is:

```text id="7x1n6r"
1. Save the current RSP.
2. Load the target PML4 into CR3.
3. Switch RSP to the target interrupt context.
4. Restore the target register state.
5. Execute iretq.
```

The ordering is significant because the target interrupt context must be accessible in the new address space.

The target stack must therefore not be selected before the target page tables are active.

## `iretq` Boundary

After the new `CR3` and stack are active, the remaining frame restoration follows the Interrupt Context Switch ABI.

The address-space switch ABI ends at the point where the target address space and target interrupt frame have been established; the final CPU execution transition is performed by:

```asm id="t9g5fh"
iretq
```

## Kernel and Userspace Address Spaces

A userspace address space is represented by its own PML4.

`AddressSpace::new_user()` creates a new PML4 and copies the kernel-only PML4 entries required to keep the kernel mapped while user mappings remain separate.

The resulting contract is:

```text id="n4n7kx"
Kernel address space
    └── kernel mappings

User address space
    ├── kernel mappings
    └── user mappings
```

The `USER` page-table flag distinguishes mappings accessible from CPL3.

## Critical Assembly Convention

### CR3 before target RSP

The essential sequence is:

```asm id="p8r1q0"
mov [rdi], rsp
mov cr3, rdx
mov rsp, rsi
```

This is deliberately ordered so that the old address space remains active while the current stack pointer is saved, and the target stack is selected only after the target page tables are active.

Changing this order can cause the target stack/context access to occur through the wrong address space.

## Boundary Summary

```text id="7x0m7d"
AddressSpace
    │
    └── pml4_address
            │
            ▼
interrupt_context_switch_to_address_space()
            │
            ├── save current RSP
            ├── CR3 = next PML4
            ├── RSP = next context
            └── restore + iretq
                    │
                    ▼
             target address space
```

The **Page Table / CR3 Address-Space Switch ABI** defines the relationship between `AddressSpace::pml4_address`, `CR3`, and the assembly-level address-space transition. It does not define individual page-table entry formats or the general virtual-memory mapping API.
