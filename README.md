# Mentacore

**Mentacore is an experimental AI operating system built from scratch in Rust.**

The long-term goal is to build an operating system designed around artificial intelligence — starting from a custom UEFI bootloader and Rust kernel, eventually providing a native Python runtime and an AI-focused kernel capable of running **Flust** directly on top of the system.

> 🚧 Mentacore is in very early development.

## Vision

Mentacore aims to build the software stack from the lowest level upward:

```text
┌─────────────────────────────────────┐
│               Flust                 │
│       AI Development System         │
├─────────────────────────────────────┤
│             AI Kernel               │
│          Python-based               │
├─────────────────────────────────────┤
│          Python Runtime             │
├─────────────────────────────────────┤
│         Mentacore Kernel            │
│              Rust                   │
├─────────────────────────────────────┤
│       Custom Rust Bootloader        │
├─────────────────────────────────────┤
│               UEFI                  │
├─────────────────────────────────────┤
│             x86_64                  │
└─────────────────────────────────────┘
```

Each layer is developed and validated before higher-level functionality is introduced.

## Architecture

The intended long-term architecture is:

```text
Flust
  ↓
AI Kernel
  ↓
Python Runtime
  ↓
Userspace
  ↓
Mentacore Kernel
  ↓
Rust Bootloader
  ↓
UEFI
  ↓
x86_64 Hardware
```

The architecture is intentionally developed from the lowest layer upward. The goal is to avoid depending on a conventional operating system for the core execution environment.

## Current Status

**Pre-alpha — Kernel infrastructure / Phase 2 development**

The basic boot and kernel infrastructure is now operational.

### Bootloader

The custom Rust UEFI bootloader currently provides:

* UEFI initialization
* Kernel loading
* UEFI memory map acquisition
* Framebuffer discovery
* Boot information construction
* Kernel handoff
* Serial debugging support

### Kernel

The kernel currently provides:

* Rust `no_std` kernel environment
* Kernel entry point
* Serial output
* Framebuffer output
* UEFI memory map parsing
* Physical frame allocator
* Physical frame bitmap
* Frame allocation and freeing
* Double-free protection
* Physical frame validation
* x86_64 page tables
* 4 KiB page mapping
* 2 MiB identity mapping
* Page unmapping
* Canonical address validation
* Physical address validation
* Kernel virtual memory layout
* Demand-paged kernel heap
* Global kernel allocator
* Multi-page heap allocation
* Initial IDT infrastructure
* Page fault handling
* Page fault diagnostics

The implemented memory-management stack has been validated with kernel-side tests covering physical allocation, paging, mapping/unmapping, and heap allocation.

Current validated milestones include:

```text
PHYSICAL FRAME FREE TEST OK
PHYSICAL FRAME REUSE TEST OK
PHYSICAL FRAME COUNTER TEST OK
PHYSICAL FRAME DOUBLE-FREE TEST OK
PHYSICAL FRAME INVALID TEST OK

PAGING DUPLICATE MAP TEST OK
PAGING MAPPING TEST OK
PAGING UNMAP TEST OK
PAGING UNMAP REJECTION TEST OK

HEAP TEST OK
HEAP MULTI-PAGE TEST OK

DISPLAY OK
```

## Development Roadmap

```text
Phase 1 — Kernel Basic Infrastructure
    ✓ UEFI memory map
    ✓ Physical frame allocator
    ✓ Page tables
    ✓ Virtual memory
    ✓ Kernel heap
    ✓ Basic framebuffer output

Phase 2 — Kernel Services
    ├── Interrupts
    ├── Exceptions
    ├── Timer
    ├── CPU management
    └── Basic device layer

Phase 3 — Process and Userspace
    ├── Processes
    ├── Address spaces
    ├── Scheduler
    ├── System calls
    └── Filesystem

Phase 4 — Python Runtime
    ├── Python runtime integration
    ├── Python execution
    ├── Import system
    ├── Required OS interfaces
    └── Runtime services

Phase 5 — AI Kernel
    ├── Python API
    ├── AI runtime layer
    ├── Memory and context
    ├── Tools
    └── AI services

Phase 6 — Flust
    ├── Flust runtime integration
    ├── Native UI APIs
    ├── Required runtime support
    └── Run Flust directly on Mentacore
```

## Development Philosophy

Mentacore is developed incrementally.

The project follows several principles:

* Build from the lowest layer upward.
* Keep every milestone independently testable.
* Validate low-level infrastructure before depending on it.
* Avoid unnecessary complexity in early kernel stages.
* Prefer explicit interfaces between system layers.
* Keep the boot process and kernel understandable.
* Use serial diagnostics extensively during low-level development.
* Document architectural decisions as the system evolves.

The goal is not simply to produce a bootable kernel, but to build a complete operating-system stack whose architecture can eventually support an AI-native development environment.

## Technology

* **Language:** Rust
* **Architecture:** x86_64
* **Boot:** UEFI
* **Kernel:** Custom `no_std` Rust kernel
* **Virtualization / Testing:** QEMU
* **Runtime:** Planned native Python runtime
* **AI Layer:** Planned Python-based AI kernel
* **Development Environment:** Flust

## Repository Structure

```text
Mentacore/
├── boot_protocol/
├── bootloader/
├── kernel/
└── scripts/
```

The `boot_protocol` crate defines the interface between the bootloader and kernel.

The `bootloader` crate is responsible for preparing the machine and transferring control to the kernel.

The `kernel` crate contains the operating-system kernel and its low-level subsystems.

The `scripts` directory contains development and QEMU execution tooling.

## Long-Term Goal

The final system is intended to look conceptually like:

```text
┌──────────────────────────────┐
│            Flust             │
├──────────────────────────────┤
│          AI Kernel           │
├──────────────────────────────┤
│       Python Runtime         │
├──────────────────────────────┤
│          Userspace           │
├──────────────────────────────┤
│      Mentacore Kernel        │
├──────────────────────────────┤
│     Rust UEFI Bootloader     │
├──────────────────────────────┤
│             UEFI             │
├──────────────────────────────┤
│         x86_64 CPU           │
└──────────────────────────────┘
```

Mentacore is currently far from this final architecture, but development is proceeding toward it layer by layer.

## License

Mentacore is released under the MIT License.
