# Syscall ABI

## Purpose

The Syscall ABI defines the contract between user-mode syscall invocation, the x86_64 syscall interrupt entry stub, and the Rust syscall dispatcher.

It specifies how syscall arguments enter the kernel, how the assembly entry point exposes the saved register state to Rust, and where the syscall result is written before returning to user mode.

The current implementation uses the `int 0x80` interrupt vector as the user-to-kernel syscall entry mechanism.

## Contract-Defining Files

### `kernel/asm/syscall.s`

Defines the machine-level syscall entry and return contract.

It specifies:

* the complete general-purpose register save order;
* the location of the CPU-generated interrupt frame;
* the arguments passed to the Rust dispatcher;
* stack alignment around the Rust call;
* restoration order;
* the final `iretq`.

### `kernel/src/syscall/context.rs`

Defines the logical syscall argument contract.

`SyscallContext` maps the syscall register state to:

```text
rax → syscall number
rdi → arg0
rsi → arg1
rdx → arg2
r10 → arg3
r8  → arg4
r9  → arg5
```

This is the register-level interface exposed to syscall handlers.

### `kernel/src/syscall/dispatch.rs`

Defines the Rust-side assembly boundary:

```rust
extern "C" fn syscall_interrupt_dispatch(
    saved_registers: *mut u64,
    cpu_frame: *mut u64,
    current_rsp: u64,
)
```

It extracts the syscall number and arguments from the assembly-defined saved-register frame, dispatches the syscall, and writes the result back into the saved `RAX` slot.

### `kernel/src/interrupts/idt.rs`

Defines the hardware entry point for syscalls.

IDT vector `0x80` is bound to `syscall_interrupt_entry` using a user-callable interrupt gate with DPL 3.

This makes the syscall entry point accessible from CPL3 while retaining the kernel code-segment target.

## Assembly Saved-Register Layout

After `syscall_interrupt_entry` saves the general-purpose registers, the frame is:

```text
+0    rbx
+8    rbp
+16   r15
+24   r14
+32   r13
+40   r12
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
+144  rsp
+152  ss
```

The first 15 entries are software-saved registers. The final five entries are the CPU-generated interrupt return frame for a CPL3-to-CPL0 transition.

## Rust Dispatch Boundary

Before calling Rust, the assembly establishes:

```text
RDI = saved-register frame
RSI = CPU interrupt frame
RDX = current kernel RSP
```

These correspond to:

```rust
syscall_interrupt_dispatch(
    saved_registers,
    cpu_frame,
    current_rsp,
)
```

The syscall dispatcher currently uses the saved-register frame for syscall arguments and result propagation.

The CPU frame and current kernel stack pointer are nevertheless part of the ABI because they are explicitly passed across the assembly/Rust boundary.

## Syscall Register Contract

The logical syscall interface is:

```text
RAX = syscall number
RDI = argument 0
RSI = argument 1
RDX = argument 2
R10 = argument 3
R8  = argument 4
R9  = argument 5
```

`RAX` serves both as the syscall number on entry and the syscall result on return.

The current syscall numbers are:

```text
0 = SYS_GET_TID
1 = SYS_USER_START
```

The numeric values are part of the userspace/kernel syscall contract.

## Result Propagation

The Rust dispatcher writes the returned `SyscallResult` into the saved `RAX` slot:

```text
saved_registers + 14 × 8
```

Because `RAX` occupies offset `112` in the saved-register frame, the normal restoration sequence eventually places the syscall result back into `RAX`.

The final `iretq` then returns to the interrupted userspace instruction with that value available to the caller.

## Critical Assembly Conventions

### Complete register preservation

The entry stub saves all general-purpose registers before entering Rust.

This allows the syscall dispatcher to inspect and modify the saved syscall state without relying on live registers across the Rust call.

### `lea rsi, [rsp + 120]`

The saved-register frame begins at `RSP`, while the CPU-created interrupt frame begins immediately after the 15 saved registers.

Therefore:

```text
RSP + 120
```

is the address of the saved `RIP` field and the beginning of the CPU return frame.

### `mov rdx, rsp`

The current kernel stack pointer is passed separately to Rust.

This preserves the exact kernel stack position at the syscall dispatch boundary without making it part of the saved-register frame.

### Stack alignment

The entry stub conditionally subtracts 8 bytes before the Rust call and restores that padding afterward.

The alignment adjustment is temporary and is not part of either the saved-register frame or the CPU return frame.

### Reverse restoration

The registers are restored in reverse order:

```text
push rax ... push rbx
        ↓
pop rbx ... pop rax
```

This restores the original register state except for the intentionally modified saved `RAX`.

### `iretq`

After register restoration, `iretq` consumes the CPU-created return frame and resumes execution at the interrupted userspace `RIP`, with the syscall result restored in `RAX`.

## Boundary Summary

```text
User mode
    │
    │ int 0x80
    ▼
IDT vector 0x80 / DPL3
    │
    ▼
syscall_interrupt_entry
    │
    ├── save GPRs
    ├── RDI = saved registers
    ├── RSI = CPU frame
    └── RDX = current kernel RSP
    │
    ▼
syscall_interrupt_dispatch(...)
    │
    ├── RAX → syscall number
    ├── RDI..R9 → syscall arguments
    └── result → saved RAX
    │
    ▼
restore GPRs
    │
    ▼
iretq
    │
    ▼
User mode
```

The syscall ABI is independent of the **Exception Entry ABI** and **Timer Interrupt ABI**, although all three ultimately rely on the x86_64 interrupt-return mechanism.
