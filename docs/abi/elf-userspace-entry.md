# ELF Userspace Entry ABI

## Purpose

The ELF Userspace Entry ABI defines the contract between the ELF loader, process/thread initialization, and the interrupt-return mechanism used to start a userspace image.

It establishes the initial userspace instruction pointer and stack pointer, together with the privilege-level state required for the first CPL3 execution.

## Contract-Defining Files

### `kernel/src/memory/elf/loader.rs`

Defines the userspace image loading contract.

The ELF loader exposes:

```rust
pub fn entry(&self) -> u64
pub fn user_stack_top(&self) -> u64
```

The ELF entry address is taken directly from the parsed ELF entry point.

The loader also defines the initial userspace stack region:

```text
USER_STACK_PAGES = 4
USER_STACK_TOP   = USER_SPACE_END aligned down to PAGE_SIZE
USER_STACK_BASE  = USER_STACK_TOP - 4 * PAGE_SIZE
```

`map_user_stack()` maps this region as writable, user-accessible, and non-executable.

### `kernel/src/process/process.rs`

Defines the process-level representation of the loaded userspace entry state.

A process created from a loaded ELF stores:

```text
entry
user_stack_top
address_space
```

The `entry` value comes from the ELF image and the stack value comes from the loader.

### `kernel/src/thread/thread.rs`

Defines how the process entry state becomes an executable userspace thread.

`Thread::new_user()` creates an `InterruptContext` with:

```text
RIP = entry
RSP = user_stack_top
```

and the appropriate userspace segment selectors and initial processor flags.

### `kernel/src/thread/interrupt_context.rs`

Defines the binary representation consumed by the interrupt-return mechanism.

`InterruptContext::new_user()` establishes the complete CPL3 return state:

```text
RIP
CS
RFLAGS
RSP
SS
```

The structure is `#[repr(C)]`, and its layout is compile-time verified.

## Userspace Entry Contract

The initial userspace state is:

```text
RIP    = ELF entry address
CS     = USER_CODE_SELECTOR
RFLAGS = INITIAL_RFLAGS
RSP    = user stack top
SS     = USER_DATA_SELECTOR
```

The address space containing the ELF image and user stack must already be active when this frame is consumed.

The userspace entry function therefore begins execution at the ELF-defined entry point with the stack established at the top of the loader-provided user stack region.

## Userspace Stack Contract

The loader reserves four pages for the initial userspace stack:

```text
USER_STACK_BASE
        │
        │ 4 pages
        ▼
USER_STACK_TOP
```

The stack is mapped with:

```text
writable = true
user     = true
executable = false
```

`user_stack_top` is the initial value assigned to the `RSP` field of `InterruptContext`.

No userspace return address is synthesized by the kernel.

## Interrupt-Return Representation

The initial userspace state is represented by:

```text
+0    r15
...
+112  rax
+120  rip
+128  cs
+136  rflags
+144  rsp
+152  ss
```

The frame is `160` bytes.

The frame is prepared before the thread is scheduled and is later consumed by the interrupt-context switching path.

The final transition to userspace is performed by `iretq`, which consumes the prepared `RIP`, `CS`, `RFLAGS`, `RSP`, and `SS` values.

## Critical Assembly Convention

The userspace entry ABI does not require a separate userspace-entry assembly routine.

The prepared `InterruptContext` is placed directly into the stack position expected by the interrupt-context switch mechanism:

```asm
mov rsp, rsi
...
iretq
```

The kernel therefore constructs the complete architectural return frame in Rust, while the assembly performs the register restoration and final `iretq`.

## Boundary Summary

```text
ELF image
   │
   ├── ELF entry
   │
   ▼
ELF loader
   │
   ├── entry
   ├── user stack mapping
   └── user_stack_top
   │
   ▼
Process
   │
   ├── address space
   ├── entry
   └── user_stack_top
   │
   ▼
Thread::new_user()
   │
   ▼
InterruptContext
   │
   ├── RIP = ELF entry
   ├── CS  = user code
   ├── RSP = user stack top
   └── SS  = user data
   │
   ▼
interrupt context switch
   │
   ▼
iretq
   │
   ▼
CPL3 ELF entry
```

The **ELF Userspace Entry ABI** defines the initial userspace execution state. It does not define the ELF file format itself, the syscall ABI, or subsequent userspace thread scheduling.
