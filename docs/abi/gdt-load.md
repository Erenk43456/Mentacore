# GDT Load / Segment Reload ABI

## Purpose

The GDT Load / Segment Reload ABI defines the low-level contract used to install the kernel's Global Descriptor Table and reload the active code and data segment registers.

It bridges the Rust GDT representation and the assembly sequence required to safely replace the active GDT and reload `CS`, `DS`, `ES`, and `SS`.

## Contract-Defining Files

### `kernel/src/cpu/gdt.rs`

Defines the GDT pointer representation and the Rust-to-assembly entry point:

```rust
unsafe extern "C" {
    fn load_gdt_and_segments(
        gdt_pointer: *const GdtPointer,
    );
}
```

`GdtPointer` is the binary operand passed to `lgdt`:

```text
limit: u16
base:  u64
```

`gdt::load()` constructs this pointer and invokes the assembly entry point.

### `kernel/asm/cpu.s`

Defines the assembly-side calling convention and CPU reload sequence.

The contract is:

```text
RDI = pointer to GdtPointer
```

The routine then:

1. Loads the supplied GDT with `lgdt`.
2. Performs a far return to reload `CS`.
3. Loads the kernel data selector into `DS`, `ES`, and `SS`.
4. Returns to Rust.

The current selector contract is:

```text
CS = 0x08
DS = 0x10
ES = 0x10
SS = 0x10
```

These selectors correspond to the kernel code/data descriptors defined by the GDT ABI.

## Critical Assembly Convention

### `lgdt`

```asm
lgdt [rdi]
```

The pointer passed in `RDI` is the exact operand consumed by the CPU's `lgdt` instruction.

### Far control transfer for `CS`

`CS` cannot be reloaded with an ordinary `mov` instruction.

The assembly therefore constructs a far-return frame:

```asm
push 0x08
lea rax, [rel .reload_cs]
push rax
retfq
```

The selector and continuation address form the architectural far-return target.

The code selector must match the kernel code descriptor installed in the new GDT.

### Data segment reload

After `CS` has been reloaded, the kernel data selector is loaded into:

```asm
mov ax, 0x10
mov ds, ax
mov es, ax
mov ss, ax
```

These values must remain synchronized with the GDT selector contract.

## Boundary Summary

```text
Rust GdtPointer
      │
      │ RDI
      ▼
load_gdt_and_segments
      │
      ├── lgdt
      ├── reload CS
      ├── reload DS/ES/SS
      └── ret
      │
      ▼
Active kernel GDT + segment state
```

The **GDT Load / Segment Reload ABI** defines the Rust ↔ assembly boundary and the required CPU reload sequence. The actual descriptor layout and selector assignments are defined separately by the GDT / Segment Selector ABI.
