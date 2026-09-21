# Low-Level Constants and ABI Documentation

This document defines the documentation approach used for Mentacore's low-level constants, binary contracts, and ABI boundaries.

It is not an implementation reference for a specific subsystem. Instead, it explains **which low-level contracts are documented, why they are documented, and how those contracts are represented across the repository**.

---

## Low-Level Contracts

Mentacore contains several interfaces where correctness depends on multiple components agreeing on a precise low-level representation.

These include, for example:

* Rust ↔ assembly interfaces;
* CPU context layouts;
* interrupt and exception frames;
* bootloader ↔ kernel handoff structures;
* boot protocol structures;
* syscall register conventions;
* hardware-defined register layouts;
* fixed selectors, offsets, and other architectural values.

These interfaces are treated as **contracts**, rather than ordinary implementation details.

A compiler can verify the Rust side of a structure, but it cannot by itself verify that an assembly routine is using the correct byte offset. Similarly, a value exchanged between the bootloader and kernel may compile correctly on both sides while still violating the binary contract between them.

The purpose of this documentation is to make those contracts explicit.

---

# ABI Documentation

Each ABI is documented separately under:

```text
docs/abi/
```

An individual ABI document describes the contract itself rather than every location that happens to use it.

For example:

```text
docs/abi/
└── context-switch.md
```

The Context Switch ABI document identifies the files that **define the contract**, explains why the contract exists, and records the important low-level conventions that must remain synchronized.

---

## Contract-Defining Files

ABI documentation distinguishes between:

### Contract-defining files

These are the files that establish the ABI itself.

Typical examples are:

* a Rust `#[repr(C)]` structure;
* an `extern "C"` declaration;
* an assembly implementation;
* a boot protocol structure;
* a hardware-facing definition;
* an initializer that establishes the required initial binary state.

These files belong in the ABI document's **Contract-Defining Files** section.

### Consumers and tests

Files that merely call, consume, or validate an ABI are not considered part of the ABI definition.

For example, a scheduler calling `context_switch()` uses the Context Switch ABI, but does not define it.

Likewise, a regression test verifying a structure's offsets protects the ABI but does not define the ABI.

This distinction keeps ABI documentation focused on the actual contract rather than becoming a list of every affected file.

---

# ABI Documentation Philosophy

Mentacore's ABI documentation follows a simple principle:

> **Document the boundary, not the entire implementation.**

An ABI document should answer:

1. **What is this ABI?**
2. **Why does Mentacore need it?**
3. **Which files define the contract?**
4. **What does each defining file contribute?**
5. **Which low-level conventions are essential to keeping both sides compatible?**

It should not reproduce an entire source file or become a tutorial for the underlying architecture.

The goal is to provide enough information for a future contributor to understand the contract before modifying either side.

---

# Why the Contract-Defining Files Matter

A low-level ABI commonly spans multiple representations.

For example:

```text
Rust structure
      │
      │ binary layout
      ▼
Assembly implementation
      │
      │ CPU state
      ▼
Hardware
```

Each layer understands the same contract differently.

The ABI documentation therefore records the files responsible for each representation.

For a Rust ↔ assembly ABI, this usually means documenting:

```text
Rust representation
        ↕
Assembly implementation
        ↕
Initial state / construction rules
```

This makes it clear where a contract originates and where its binary assumptions are implemented.

---

# Assembly Documentation

Assembly code should not be documented instruction-by-instruction.

Only instructions or patterns that are significant to the ABI should receive a short explanation.

For example:

```asm
mov [rdi + 24], rbx
```

should be explained when the important fact is that:

```text
+24 = KernelContext.rbx
```

The documentation should explain **why the offset exists**, not teach the `mov` instruction itself.

Similarly:

```asm
mov rax, [rsi + 8]
jmp rax
```

is worth documenting when it establishes that the target `RIP` is loaded from a defined context field and execution is transferred directly to it.

The purpose is to preserve the reasoning behind the assembly's binary assumptions.

---

# ABI vs. Implementation Detail

Not every low-level constant or assembly instruction constitutes an ABI.

A value belongs in ABI documentation when another component depends on its exact representation or semantics.

For example:

```text
Rust field offset
assembly register mapping
bootloader structure layout
interrupt frame layout
syscall register assignment
segment selector shared across components
```

are contract-level details.

By contrast, an internal temporary register choice or an implementation detail with no cross-boundary dependency generally does not need to be documented as part of an ABI.

---

# Documentation Scope

ABI documents should remain intentionally focused.

A document should generally contain:

```text
ABI identity
    ↓
Purpose
    ↓
Contract-defining files
    ↓
Role of each defining file
    ↓
Critical binary / assembly conventions
```

The document does not need to enumerate:

* every caller;
* every test;
* every affected module;
* every implementation detail;
* every assembly instruction.

Those belong in source code, subsystem documentation, tests, or separate design documents where appropriate.

---

# Current ABI Documentation

The ABI documentation currently includes:

| ABI                      | Document                                                |
| ------------------------ | ------------------------------------------------------- |
| Kernel context switching | [`docs/abi/context-switch.md`](./abi/context-switch.md) |

Additional ABI documents should be added as individual low-level contracts are identified and verified.

The list is intentionally maintained here so that this document acts as the **index and methodology reference** for Mentacore's ABI documentation.

---

# Recommended ABI Categories

As Mentacore's low-level architecture evolves, ABI documentation may cover contracts such as:

* kernel context switching;
* interrupt context switching;
* interrupt/exception entry frames;
* syscall entry and return;
* thread startup;
* address-space/CR3 transitions;
* kernel entry from the bootloader;
* boot protocol structures;
* userspace entry;
* other Rust ↔ assembly boundaries.

These should remain separate documents when they represent distinct binary contracts.

---

# Change Philosophy

ABI changes should be treated differently from ordinary refactoring.

A seemingly small change to:

* a structure field;
* a field order;
* a register assignment;
* an assembly offset;
* a stack layout;
* a selector;
* a calling convention;

can silently invalidate another component.

Therefore, when a low-level contract changes:

1. Identify every contract-defining side.
2. Verify that all representations still agree.
3. Update the corresponding ABI document.
4. Update the relevant regression coverage.
5. Run the normal Mentacore validation and regression process.

The documentation should describe the **intentional contract**, not preserve historical implementation accidents.

---

# Guiding Principle

Mentacore's low-level documentation exists to preserve the reasoning behind binary interfaces.

The objective is not to document every low-level line of code.

The objective is to ensure that when two components depend on an exact machine-level contract, that contract is:

* explicit;
* discoverable;
* independently understandable;
* tied to its defining source files;
* protected by regression coverage where appropriate.

This makes low-level changes deliberate rather than dependent on rediscovering undocumented assumptions.
