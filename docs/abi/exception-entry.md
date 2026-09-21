# Exception Entry ABI

## Purpose

The **Exception Entry ABI** defines the contract between x86_64 exception entry stubs and the Rust exception dispatch layer.

It exists because the CPU provides part of the exception frame automatically, while Mentacore's assembly entry points save additional registers and normalize exception-specific state before calling Rust.

The ABI therefore defines exactly how an exception reaches its Rust dispatcher and how the dispatcher interprets the resulting `TrapFrame`.

---

## Contract-Defining Files

### `kernel/src/interrupts/mod.rs`

**Rust-side exception frame definition.**

Defines `TrapFrame` with the exact binary layout consumed by the exception dispatch functions:

```text
+0    r11
+8    r10
+16   r9
+24   r8
+32   rdi
+40   rsi
+48   rdx
+56   rcx
+64   rax
+72   error_code
+80   rip
+88   cs
+96   rflags
```

The file also contains compile-time size and offset assertions that enforce this layout.

---

### `kernel/asm/exceptions.s`

**Assembly-side exception entry implementation.**

Defines the exception entry points:

```text
divide_error_entry
invalid_opcode_entry
double_fault_entry
general_protection_entry
page_fault_entry
```

These stubs:

1. disable interrupts;
2. save the required general-purpose registers;
3. normalize the error-code position;
4. pass the resulting frame to Rust;
5. restore the saved state when the exception path returns;
6. complete the return with `iretq` where applicable.

For exceptions that do not provide a CPU error code, the entry stub pushes a synthetic zero so that the Rust dispatcher receives a consistent `TrapFrame` error-code position.

---

### `kernel/src/interrupts/exceptions.rs`

**Rust-side dispatch interface.**

Defines the exception dispatch functions and their `extern "C"` ABI:

```text
divide_error_dispatch(trap_frame)
invalid_opcode_dispatch(trap_frame)
double_fault_dispatch(trap_frame, error_code, cpu_rsp)
general_protection_dispatch(trap_frame, error_code)
page_fault_dispatch(trap_frame, error_code, current_rsp)
```

These signatures define what the assembly entry stubs must provide.

The additional arguments for double fault and page fault are part of their respective entry contracts.

---

### `kernel/src/interrupts/idt.rs`

**Exception entry registration side of the ABI.**

Defines the IDT entries that connect CPU exception vectors to the assembly entry points.

The relevant mappings include:

```text
#DE  → divide_error_entry
#UD  → invalid_opcode_entry
#DF  → double_fault_entry
#GP  → general_protection_entry
#PF  → page_fault_entry
```

It also defines the gate configuration and, for double fault, the use of IST1.

---

## Exception Frame Contract

After the assembly register pushes, the Rust `TrapFrame` is expected to appear as:

```text
+0    r11
+8    r10
+16   r9
+24   r8
+32   rdi
+40   rsi
+48   rdx
+56   rcx
+64   rax
+72   error_code
+80   rip
+88   cs
+96   rflags
```

For exceptions without a CPU-provided error code:

```text
error_code = 0
```

is inserted by the assembly entry stub.

For exceptions with a CPU-provided error code, the existing CPU value occupies the same position.

This normalization allows the Rust dispatcher to use one `TrapFrame` representation.

---

## Critical Assembly Conventions

### Synthetic error code

For exceptions such as `#DE` and `#UD`, the CPU does not provide an error code.

The entry stub therefore performs:

```asm
push 0
```

after saving the general-purpose registers.

This keeps `TrapFrame.error_code` at the same offset for both error-code and non-error-code exceptions.

---

### `RDI = TrapFrame`

The assembly establishes:

```asm
mov rdi, rsp
```

after constructing the frame.

The Rust dispatcher therefore receives the address of the normalized `TrapFrame` as its first argument.

---

### Error-code extraction

For exceptions with a CPU-provided error code:

```asm
mov rsi, [rdi + CPU_ERROR_CODE_OFFSET]
```

The error code is passed separately to Rust while remaining part of the `TrapFrame`.

This provides both the complete frame and a direct dispatch argument.

---

### Double-fault CPU RSP

The double-fault entry additionally captures the stack pointer associated with the pre-IST state:

```asm
mov rax, rsp
```

and later passes the saved value as the third dispatch argument.

This is specific to the double-fault/IST contract and is not part of the normal exception frame.

---

### Page-fault current RSP

The page-fault entry passes its current frame address separately:

```asm
mov rdx, rsp
```

The Rust page-fault dispatcher uses this value when handling a user-space page fault and terminating the current user thread.

---

### Call alignment

The exception stubs check:

```asm
test rsp, 8
```

and conditionally reserve eight bytes before calling Rust.

This preserves the stack alignment required by the Rust/C-compatible function-call boundary without changing the logical `TrapFrame` layout.

---

### `iretq`

The page-fault path restores the saved registers and finishes with:

```asm
iretq
```

The CPU exception return frame remains below the software-saved registers, allowing `iretq` to restore the interrupted execution state.

Exception paths that terminate permanently do not reach this return sequence.

---

## ABI Boundary

```text
CPU exception
     │
     │ CPU exception frame
     ▼
kernel/asm/exceptions.s
     │
     │ software-saved registers
     │ normalized error code
     ▼
TrapFrame
     │
     │ extern "C" dispatch arguments
     ▼
kernel/src/interrupts/exceptions.rs
```

`kernel/src/interrupts/idt.rs` establishes which CPU vectors enter these assembly stubs.

Together, these four files define the **Exception Entry ABI**.
