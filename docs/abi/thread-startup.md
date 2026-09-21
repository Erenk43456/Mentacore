# Thread Startup ABI

## Purpose

The Thread Startup ABI defines how a newly created kernel or userspace thread is represented so that the existing context-switch and interrupt-return mechanisms can start executing it as if it had already been running.

It defines the initial stack position, instruction pointer, processor flags, privilege level, and interrupt-frame representation required for first execution.

## Contract-Defining Files

### `kernel/src/thread/thread.rs`

Defines how new threads are initialized.

For kernel threads, `ThreadEntry` is:

```rust id="4x8m3n"
pub type ThreadEntry = extern "C" fn() -> !;
```

The initial `KernelContext` is created with:

```text id="0y5q8p"
RSP = kernel_stack.top() - 8
RIP = entry
RFLAGS = INITIAL_RFLAGS
```

The reserved stack slot establishes the stack alignment expected by the kernel thread entry ABI.

For kernel interrupt startup, `prepare_interrupt_context()` creates a `KernelInterruptContext` containing the initial `RIP`, kernel `CS`, and initial `RFLAGS`.

For userspace threads, `new_user()` creates an `InterruptContext` containing the userspace entry `RIP`, userspace stack `RSP`, user code/data selectors, and initial `RFLAGS`.

### `kernel/src/thread/context.rs`

Defines the initial kernel execution context through `KernelContext`.

The representation is:

```text id="0v0xgc"
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

The context-switch assembly consumes this exact representation when the new thread is first selected.

### `kernel/src/thread/interrupt_context.rs`

Defines the interrupt-return frames used to start kernel and userspace threads through the `iretq` path.

`KernelInterruptContext` represents a CPL0 return frame.

`InterruptContext` represents a CPL3 return frame and additionally contains:

```text id="1g8rmb"
rsp
ss
```

The compile-time layout assertions make these offsets part of the contract.

### `kernel/asm/context_switch.s`

Defines how the initial `KernelContext` and interrupt contexts are consumed.

A kernel thread begins execution through the `KernelContext` path, while an interrupt-context switch restores the prepared interrupt frame and terminates in `iretq`.

## Kernel Thread Startup

A kernel thread is initialized with:

```text id="g8up0f"
KernelContext
    │
    ├── RSP = kernel_stack.top() - 8
    ├── RIP = ThreadEntry
    └── RFLAGS = INITIAL_RFLAGS
```

When the context is selected, `context_switch` restores `RSP`, registers, and `RFLAGS`, loads `RIP`, and jumps directly to the entry function.

There is no synthetic return address. The entry function has type:

```rust id="c9c8m4"
extern "C" fn() -> !
```

and therefore is not expected to return.

## Kernel Interrupt Startup

Kernel threads also receive an initial `KernelInterruptContext`.

Its contract is:

```text id="u8f7f7"
+0    r15
...
+112  rax
+120  rip
+128  cs
+136  rflags
```

The initial frame uses:

```text id="4n4z6n"
RIP    = thread entry
CS     = KERNEL_CODE_SELECTOR
RFLAGS = INITIAL_RFLAGS
```

This frame is positioned so that the interrupt-context switch machinery can restore the saved registers and execute `iretq`.

## Userspace Thread Startup

A userspace thread receives a full `InterruptContext`:

```text id="uvq5m3"
+0    r15
...
+112  rax
+120  rip
+128  cs
+136  rflags
+144  rsp
+152  ss
```

The initial values include:

```text id="n4k0bl"
RIP    = userspace entry
CS     = USER_CODE_SELECTOR
RFLAGS = INITIAL_RFLAGS
RSP    = userspace stack top
SS     = USER_DATA_SELECTOR
```

This provides the complete CPU return state required for `iretq` to transition from CPL0 to CPL3.

## Stack Alignment Contract

Kernel thread startup deliberately uses:

```text id="7ykh1n"
kernel_stack.top() - 8
```

rather than the raw stack top.

The initial stack position must satisfy the calling convention expected by the `extern "C"` thread entry function when `context_switch` transfers execution directly to it.

The reserved slot is therefore part of the startup contract, not an arbitrary stack adjustment.

## Critical Assembly Conventions

### Direct `RIP` restoration

`context_switch` loads the prepared `RIP` and executes:

```asm id="v1n9jy"
mov rax, [rsi + 8]
jmp rax
```

This starts a never-before-run thread without requiring a synthetic call frame.

### Prepared interrupt frame

`interrupt_context_switch` treats the `InterruptContext` as an already constructed interrupt-return frame:

```asm id="w8v4by"
mov rsp, rsi
...
iretq
```

The assembly does not construct the CPU return state itself. The thread initialization code constructs the frame that `iretq` will consume.

### Privilege transition through `iretq`

For a userspace thread, the prepared `CS`, `RIP`, `RFLAGS`, `RSP`, and `SS` fields cause `iretq` to restore the complete CPL3 execution state.

## Boundary Summary

```text id="2g6z3v"
Thread creation
    │
    ├── kernel thread
    │      └── KernelContext
    │             ├── RSP
    │             ├── RIP
    │             └── RFLAGS
    │
    └── userspace thread
           └── InterruptContext
                  ├── RIP
                  ├── CS
                  ├── RFLAGS
                  ├── RSP
                  └── SS
    │
    ▼
Scheduler selects thread
    │
    ▼
context_switch / interrupt_context_switch
    │
    ▼
first execution
```

The Thread Startup ABI defines the **initial state** of a thread. The Context Switch ABI defines how an already-established `KernelContext` is subsequently saved and restored.
