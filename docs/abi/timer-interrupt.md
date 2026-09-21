# Timer Interrupt ABI

## Purpose

The Timer Interrupt ABI defines the contract between x86_64 timer interrupt entry stubs and the Rust timer dispatch functions.

It guarantees that both the legacy PIT timer IRQ and the LAPIC timer IRQ enter Rust with the same saved-register layout, a stable dispatch argument, a correctly aligned call stack, and a well-defined `iretq` return path.

The two timer sources therefore share one entry-frame ABI while retaining separate Rust dispatch functions.

## Contract-Defining Files

### `kernel/asm/timer.s`

Defines the machine-level timer interrupt entry contract.

It specifies:

* which general-purpose registers are saved;
* the exact order and therefore the memory layout of the saved frame;
* the register used to pass the frame pointer to Rust;
* stack alignment before the Rust call;
* restoration order;
* the final `iretq` return.

Both `timer_irq_entry` and `lapic_timer_entry` implement the same frame contract.

### `kernel/src/interrupts/timer.rs`

Defines the Rust side of the dispatch contract.

The assembly entry points call:

```rust
extern "C" fn timer_irq_dispatch(dispatch_rsp: u64)
extern "C" fn lapic_timer_dispatch(dispatch_rsp: u64)
```

`dispatch_rsp` is the address of the saved-register frame created by `timer.s`.

The LAPIC dispatch path additionally interprets the interrupted CPU state and may transfer execution through the interrupt-context-switch ABI when user-mode preemption is enabled.

### `kernel/src/interrupts/idt.rs`

Defines the hardware-to-entry-point portion of the contract.

The IDT binds:

* vector `32` → `timer_irq_entry`;
* `LAPIC_TIMER_VECTOR` (`0x40`) → `lapic_timer_entry`.

Both entries use the kernel code segment and an interrupt gate with IST index `0`.

## Saved Register Layout

At the point where Rust is called, `RDI` contains the address of this frame:

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
```

The interrupted CPU frame remains immediately above this software-saved register frame and is the frame consumed by `iretq`.

The timer dispatch functions therefore receive a pointer to the saved-register frame rather than a Rust struct representing the complete hardware interrupt frame.

## Critical Assembly Conventions

### `mov rdi, rsp`

The saved-register frame begins at the current `RSP`. Moving `RSP` into `RDI` establishes the first System V AMD64 argument for the Rust `extern "C"` dispatch function.

### Stack alignment before `call`

```asm
test rsp, 8
jz .dispatch_aligned
sub rsp, 8
call timer_irq_dispatch
add rsp, 8
```

The temporary subtraction is not part of the saved interrupt frame. It exists only to satisfy the ABI stack-alignment requirement for the Rust function call.

### Reverse-order restoration

The registers are restored in the exact reverse order of their pushes:

```text
push rax ... push r15
        ↓
pop r15 ... pop rax
```

This preserves the original register values and restores `RSP` to the hardware interrupt frame.

### `iretq`

After all software-saved registers have been restored, `iretq` consumes the CPU-created interrupt frame and resumes the interrupted execution context.

## Timer Sources

The ABI is shared by two hardware sources:

```text
Legacy PIT
  IDT vector 32
      ↓
timer_irq_entry
      ↓
timer_irq_dispatch(dispatch_rsp)

LAPIC timer
  IDT vector 0x40
      ↓
lapic_timer_entry
      ↓
lapic_timer_dispatch(dispatch_rsp)
```

The entry-frame layout is identical for both paths. Their Rust dispatch behavior differs after the ABI boundary.

## Boundary Summary

The contract can be reduced to:

```text
Hardware interrupt
    ↓
IDT vector
    ↓
timer_irq_entry / lapic_timer_entry
    ↓
15-register software frame
    ↓
RDI = frame address
    ↓
Rust extern "C" dispatch
    ↓
register restoration
    ↓
iretq
```

The LAPIC dispatch path may subsequently enter the separate **Interrupt Context Switch ABI** when preemption selects another user address space/thread.
