# Mentacore

**Mentacore is an experimental AI operating system built from scratch in Rust.**

The long-term goal of Mentacore is to build an operating system designed around artificial intelligence — starting from a custom bootloader and kernel, eventually providing a native Python runtime and an AI-focused kernel capable of running **Flust** directly on top of the system.

> 🚧 Mentacore is in very early development.

## Vision

Mentacore aims to build the entire software stack from the ground up:

```text
┌─────────────────────────────────────┐
│               Flust                 │
│        AI Development System        │
├─────────────────────────────────────┤
│        Python-based AI Kernel       │
├─────────────────────────────────────┤
│          Python Runtime             │
├─────────────────────────────────────┤
│         Mentacore Kernel            │
│               Rust                  │
├─────────────────────────────────────┤
│        Custom Rust Bootloader       │
├─────────────────────────────────────┤
│              UEFI                   │
├─────────────────────────────────────┤
│             x86_64                  │
└─────────────────────────────────────┘
```

The project is intentionally developed from the lowest level upward. Each layer will be implemented and validated before building the next one.

## Long-Term Goals

### 1. Custom Bootloader

Build a custom x86_64 UEFI bootloader entirely in Rust.

Responsibilities will eventually include:

* UEFI initialization
* Kernel loading
* Memory map acquisition
* CPU and execution environment setup
* Kernel handoff
* Boot information passing

### 2. Custom Kernel

Build the Mentacore kernel from scratch in Rust.

Planned subsystems include:

* Memory management
* Physical and virtual memory
* Paging
* Interrupt handling
* CPU management
* Process and thread management
* Scheduling
* Device management
* Filesystem support
* Userspace

### 3. Native Python Runtime

Once the Rust kernel provides a sufficiently complete operating environment, Mentacore will introduce a Python runtime directly on top of the kernel.

The goal is to make Python a first-class environment rather than merely running an application on top of a conventional operating system.

### 4. AI Kernel

On top of the Python runtime, Mentacore will develop a Python-based AI kernel.

The AI kernel is intended to become the primary high-level intelligence and orchestration layer of the operating system.

### 5. Flust

The final long-term goal is to run **Flust** directly on Mentacore.

Flust will act as the development and intelligence environment operating on top of the AI kernel.

The intended architecture is:

```text
Flust
  ↓
AI Kernel
  ↓
Python Runtime
  ↓
Mentacore Kernel
  ↓
Rust Bootloader
  ↓
UEFI
  ↓
x86_64 Hardware
```

## Development Philosophy

Mentacore is developed incrementally.

The project intentionally starts with the smallest possible bootable system and adds complexity only after each previous layer has been verified.

The initial milestones will therefore be deliberately simple:

```text
UEFI
 ↓
Bootloader
 ↓
Kernel
 ↓
Kernel output
 ↓
Memory management
 ↓
Interrupts
 ↓
Processes
 ↓
Userspace
 ↓
Python Runtime
 ↓
AI Kernel
 ↓
Flust
```

This approach makes low-level failures easier to isolate and keeps the architecture understandable as the system grows.

## Technology

* **Language:** Rust
* **Architecture:** x86_64
* **Boot:** UEFI
* **Kernel:** Custom
* **Runtime:** Planned Python runtime
* **AI Layer:** Planned Python-based AI kernel
* **Development Environment:** Flust

## Current Status

🚧 **Pre-alpha / Early development**

The project is currently focused on establishing the fundamental boot and kernel execution path.

Many components described in the long-term architecture have not yet been implemented.

## License

Mentacore is released under the MIT License.
