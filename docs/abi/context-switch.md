# Context Switch ABI

## Purpose

The **Context Switch ABI** defines the low-level contract used to transfer CPU execution from one kernel thread context to another.

It exists because the scheduler operates in Rust, while the actual CPU register and stack transition is implemented in x86_64 assembly. Both sides must therefore agree on how a `KernelContext` is represented and how its address is passed to the assembly routine.

The ABI is intentionally small: Rust provides the context representation and function declaration, while assembly interprets that representation and performs the CPU-level transition.

---

## Contract-Defining Files

The ABI is defined by the following files:

### `kernel/src/thread/context.rs`

**Rust side of the ABI.**

Defines `KernelContext` and its binary representation:

```rust
#[repr(C)]
pub struct KernelContext {
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}
```

It also declares the assembly entry point:

```rust
unsafe extern "C" {
    pub fn context_switch(
        current: *mut KernelContext,
        next: *const KernelContext,
    );
}
```

This file therefore defines **what data crosses the ABI boundary and how the assembly function is exposed to Rust**.

---

### `kernel/asm/context_switch.s`

**Assembly side of the ABI.**

Implements `context_switch` and interprets `KernelContext` using fixed byte offsets.

The implementation expects:

```text
RDI = current
RSI = next
```

and therefore uses:

```asm
[rdi + offset]
```

for the current context and:

```asm
[rsi + offset]
```

for the next context.

The assembly-side offsets correspond directly to the Rust structure:

```text
+0   rsp
+8   rip
+16  rflags
+24  rbx
+32  rbp
+40  r12
+48  r13
+56  r14
+64  r15
```

This file defines **how the binary context representation is interpreted and how CPU execution is transferred between contexts**.

---

### `kernel/src/thread/thread.rs`

**Initial-context side of the ABI.**

This file establishes the initial `KernelContext` values used when a kernel thread has not previously executed.

A new thread is initialized with:

```text
RSP = kernel_stack.top() - 8
RIP = thread entry function
```

This is part of the context-switch contract because `context_switch` restores `RSP` and jumps directly to `RIP`.

It also defines the kernel thread entry type:

```rust
pub type ThreadEntry = extern "C" fn() -> !;
```

Therefore this file defines **what constitutes a valid initial context for a newly created kernel thread**.

---

## Critical Assembly Conventions

Only the instructions that directly define the ABI are documented here.

### `RDI` / `RSI`

```asm
; RDI = current
; RSI = next
```

These registers carry the two `context_switch` arguments from Rust into assembly.

---

### Context offsets

```asm
mov [rdi + 24], rbx
```

`+24` is the `rbx` field of `KernelContext`.

The assembly uses numeric offsets because it has no knowledge of Rust field names.

The Rust structure layout and these offsets must therefore remain synchronized.

---

### Saving `RSP`

```asm
lea rax, [rsp + 8]
mov [rdi + 0], rax
```

`[rsp]` contains the return address of the current `context_switch` call.

The saved context must point past that address, so `rsp + 8` becomes the stored context stack pointer.

---

### Saving `RIP`

```asm
mov rax, [rsp]
mov [rdi + 8], rax
```

The return address at `[rsp]` becomes the saved instruction pointer.

When this context is restored later, execution can continue from that address.

---

### Restoring `RSP`

```asm
mov rsp, [rsi + 0]
```

Loads the target thread's saved stack pointer.

From this point onward, the assembly is operating on the target context's stack.

---

### Restoring `RIP`

```asm
mov rax, [rsi + 8]
jmp rax
```

Loads the target instruction pointer and transfers execution directly to it.

`jmp` is used because the target context already contains both its own `RSP` and `RIP`; the transition is not implemented as a normal `call`/`ret` sequence.

---

## ABI Boundary

```text
Rust
kernel/src/thread/context.rs
        │
        │ KernelContext
        │ extern "C" context_switch()
        ▼
Assembly
kernel/asm/context_switch.s
        │
        │ CPU context transition
        ▼
Thread execution
```

The initial context contract is established by:

```text
kernel/src/thread/thread.rs
```

Together, these files define the **Context Switch ABI**.

Files such as the scheduler and tests consume or validate this ABI, but they do not define the ABI itself.
