# Interrupt Context Switch ABI

## Purpose

The **Interrupt Context Switch ABI** defines the low-level contract used to resume kernel or userspace execution from an interrupt context.

Unlike the normal kernel context-switch ABI, which restores a `KernelContext` and transfers control with `jmp`, this ABI operates on an interrupt return frame and completes the transition with `iretq`.

This contract exists because interrupt-driven scheduling does not preserve execution state in the same representation as a normal voluntary context switch. The CPU and Mentacore's interrupt entry path construct an interrupt frame, and the context-switch assembly must interpret that frame with the exact expected layout.

The ABI also provides a variant that changes the target address space before restoring the target interrupt context.

---

## Contract-Defining Files

The ABI is defined by the following files.

### `kernel/src/thread/interrupt_context.rs`

**Rust-side representation of the interrupt return frame.**

Defines `InterruptContext` and its binary layout used by the assembly context-switch routines.

The structure represents the register state that must be restored before returning from an interrupt.

Its layout corresponds to the stack consumed by the assembly implementation.

The contract distinguishes between:

* the register state restored by the assembly `pop` sequence;
* the CPU interrupt-return frame consumed by `iretq`.

---

### `kernel/src/thread/context.rs`

**Rust-side declaration of the assembly interface.**

Declares:

```rust
unsafe extern "C" {
    pub fn interrupt_context_switch(
        current_rsp: *mut u64,
        next: *const u64,
    ) -> !;

    pub fn interrupt_context_switch_to_address_space(
        current_rsp: *mut u64,
        next: *const u64,
        next_pml4: u64,
    ) -> !;
}
```

This defines the Rust-to-assembly boundary and the arguments supplied to both assembly entry points.

The `!` return type reflects the control-flow contract: these routines restore a target interrupt frame and terminate through `iretq` rather than returning normally to their Rust caller.

---

### `kernel/asm/context_switch.s`

**Assembly implementation of the interrupt context-switch ABI.**

Defines:

```text
interrupt_context_switch
interrupt_context_switch_to_address_space
```

Both routines consume the same interrupt-context frame layout.

The second variant additionally accepts a target PML4 physical address and performs the CR3 transition before restoring the target stack.

---

## Interrupt Frame Contract

The assembly expects the target interrupt context to have the following layout:

```text
+0    r15
+8    r14
+16   r13
+24   r12
+32   rbp
+40   rbx
+48   r11
+56   r10
+64   r9
+72   r8
+80   rdi
+88   rsi
+96   rdx
+104  rcx
+112  rax
+120  rip
+128  cs
+136  rflags
+144  rsp      (CPL3 return)
+152  ss       (CPL3 return)
```

The first fifteen entries are restored explicitly by assembly.

The remaining CPU return frame is consumed by `iretq`.

The distinction between the `+144/+152` fields and the kernel-only return case is part of the interrupt-return contract.

---

## Assembly Register Mapping

For:

```text
interrupt_context_switch(current_rsp, next)
```

the assembly receives:

```text
RDI = current_rsp
RSI = next
```

For:

```text
interrupt_context_switch_to_address_space(
    current_rsp,
    next,
    next_pml4
)
```

the mapping is:

```text
RDI = current_rsp
RSI = next
RDX = next_pml4
```

The first argument identifies where the currently active interrupt stack pointer must be stored.

The second identifies the target interrupt frame.

The third is used only by the address-space-switching variant.

---

## Saving the Current Interrupt Stack

The current stack pointer is saved with:

```asm
mov [rdi], rsp
```

Unlike the normal `KernelContext` ABI, there is no separate `rsp` field inside the interrupt frame.

The interrupt context itself already exists on the stack.

Therefore the saved value is the address of the current interrupt frame:

```text
current_rsp → current interrupt frame
```

This allows the scheduler to preserve the exact frame that must later be resumed.

---

## Loading the Target Interrupt Context

The target stack is selected with:

```asm
mov rsp, rsi
```

At this point:

```text
RSP = address of target InterruptContext
```

The subsequent `pop` instructions therefore consume the target context directly from memory.

The sequence:

```asm
pop r15
pop r14
pop r13
pop r12
pop rbp
pop rbx
pop r11
pop r10
pop r9
pop r8
pop rdi
pop rsi
pop rdx
pop rcx
pop rax
```

must remain exactly consistent with the interrupt-frame layout.

---

## `iretq`

The final instruction is:

```asm
iretq
```

`iretq` is required because the remaining frame contains the architectural interrupt-return state:

```text
RIP
CS
RFLAGS
RSP
SS
```

when returning to CPL3.

Unlike `ret`, `iretq` restores the privilege-sensitive CPU state required for an interrupt return.

This is the fundamental distinction between this ABI and the normal `context_switch` ABI.

---

# Address-Space Variant

`interrupt_context_switch_to_address_space` extends the same interrupt context contract with a CR3 transition.

Its additional argument is:

```text
RDX = next PML4 physical address
```

The critical sequence is:

```asm
mov [rdi], rsp
mov cr3, rdx
mov rsp, rsi
```

The ordering is intentional.

The current interrupt stack is saved while the current address space is still active.

Only then is CR3 changed.

After the address-space transition:

```asm
mov rsp, rsi
```

selects the target thread's interrupt frame.

The remainder of the operation is identical to the normal interrupt-context switch and ends with `iretq`.

---

## ABI Variants

The two assembly entry points share the same interrupt-frame contract:

```text
InterruptContext
       │
       ├── interrupt_context_switch
       │
       └── interrupt_context_switch_to_address_space
```

The difference is the address-space operation.

### `interrupt_context_switch`

```text
save current RSP
       ↓
load target RSP
       ↓
restore registers
       ↓
iretq
```

### `interrupt_context_switch_to_address_space`

```text
save current RSP
       ↓
load CR3
       ↓
load target RSP
       ↓
restore registers
       ↓
iretq
```

The frame layout itself does not change between the two variants.

---

## Critical Assembly Conventions

### `mov [rdi], rsp`

Stores the currently active interrupt-frame address.

The ABI passes a pointer to the storage location rather than embedding the current stack pointer into `InterruptContext`.

---

### `mov rsp, rsi`

Makes the target interrupt frame the active stack.

This allows the subsequent `pop` instructions to restore the target register state directly.

---

### `pop r15` ... `pop rax`

The pop order is part of the ABI.

Each `pop` corresponds to one field in the interrupt frame.

Changing the order without changing the Rust representation breaks the binary contract.

---

### `mov cr3, rdx`

Used only by the address-space variant.

`RDX` contains the physical address of the target PML4.

The CR3 update must occur while the current stack is still usable under the current address space.

---

### `iretq`

Terminates the ABI transition by restoring the architectural interrupt-return frame.

It restores the target execution point and, when applicable, the target privilege level and userspace stack.

---

## ABI Boundary

```text
Rust
kernel/src/thread/interrupt_context.rs
        │
        │ InterruptContext representation
        ▼
Rust
kernel/src/thread/context.rs
        │
        │ extern "C" declarations
        ▼
Assembly
kernel/asm/context_switch.s
        │
        │ register restoration + iretq
        ▼
CPU execution state
```

For the address-space variant, the same boundary additionally carries:

```text
next_pml4
    │
    ▼
CR3
```

Together, these files define the **Interrupt Context Switch ABI**.
