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
│               Rust                  │
├─────────────────────────────────────┤
│        Custom UEFI Bootloader       │
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
Rust UEFI Bootloader
  ↓
UEFI
  ↓
x86_64 Hardware
```

The architecture is intentionally developed from the lowest layer upward. The goal is to eventually provide a complete execution environment without depending on a conventional operating system for the core system stack.

## Current Status

**Pre-alpha — Phase 3: Processes and Userspace**

The system currently boots a custom Rust kernel through a custom UEFI bootloader and provides a functioning low-level kernel foundation.

Phase 1 established the boot, memory-management, virtual-memory, heap, and framebuffer foundations.

Phase 2 established CPU state management, interrupt and exception infrastructure, hardware timer support, and interrupt-safe kernel synchronization.

Phase 3 is now focused on introducing processes, threads, address spaces, scheduling, system calls, and userspace infrastructure.

## Bootloader

The custom Rust UEFI bootloader currently provides:

* UEFI initialization
* Kernel ELF loading
* PT_LOAD segment validation and loading
* Kernel memory allocation
* Kernel stack allocation
* UEFI memory map acquisition
* Framebuffer discovery
* Preferred GOP mode selection
* Boot information construction
* Kernel handoff
* Exit from UEFI boot services
* Serial diagnostics

The bootloader transfers control directly to the kernel after constructing the boot information structure and leaving UEFI boot services.

## Kernel

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
* GDT initialization
* CPU state initialization
* Task State Segment (TSS)
* Kernel stack configuration
* IST1 stack configuration
* 256-entry IDT
* Interrupt handler registration
* Page fault handling
* Double fault handling with IST1
* LAPIC initialization
* LAPIC MMIO mapping
* PIT initialization
* Hardware interrupt support
* Timer interrupt handling
* TSC calibration
* Interrupt state save and restore
* Kernel spinlock
* Interrupt-safe spinlock locking
* Synchronized physical frame allocator access from exception context

## Kernel Initialization Architecture

Kernel startup is organized into dedicated initialization modules rather than keeping all boot logic inside `main.rs`.

```text
kernel/src/boot/
├── memory.rs
├── paging.rs
├── heap.rs
├── cpu.rs
├── lapic.rs
├── interrupts.rs
├── interrupt_controllers.rs
└── display.rs
```

The kernel entry point acts primarily as an orchestration layer, while subsystem-specific initialization remains isolated.

This structure is intended to make the kernel easier to extend as processes, address spaces, scheduling, and userspace are introduced.

## Kernel Test Infrastructure

Mentacore includes a custom kernel-side test infrastructure designed specifically for `no_std` execution.

The test system provides:

* Custom `TestRunner` abstraction
* Modular kernel test suites
* Serial-based test reporting
* Test pass/fail tracking
* Aggregate test results
* QEMU-based automated execution
* Automated timeout handling
* Kernel-reported failure detection
* Automated `ALL TESTS PASSED` detection

The test infrastructure is enabled separately through the `kernel-tests` Cargo feature and does not depend on the standard Rust test harness.

### Test Suites

The current test suites cover:

* Physical frame allocation
* Paging and address spaces
* Heap allocation
* Interrupts and exceptions
* Synchronization primitives
* Timer behavior
* TSC calibration

The complete kernel test suite currently validates:

```text
25/25 TESTS PASSED
ALL TESTS PASSED
```

Current validated tests include:

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
PAGING USER MAPPING PERMISSIONS TEST OK
PAGING VIRTUAL ADDRESS LAYOUT TEST OK
ADDRESS SPACE ABSTRACTION TEST OK
ADDRESS SPACE MAPPING TEST OK
ADDRESS SPACE UNMAPPING TEST OK

HEAP TEST OK
HEAP MULTI-PAGE TEST OK

INTERRUPT STATE TEST OK
TRAP FRAME LAYOUT TEST OK
TIMER STACK ALIGNMENT TEST OK
TIMER STABILITY TEST OK
LAPIC TIMER STACK ALIGNMENT TEST OK
DOUBLE FAULT IST1 TEST OK

SPINLOCK TEST OK
SPINLOCK INTERRUPT-SAFE TEST OK

TSC CALIBRATION OK
```

The test infrastructure is intentionally implemented inside the kernel so low-level subsystems can be validated in the actual `no_std` execution environment.

## QEMU Test Runner

A dedicated QEMU test runner is provided separately from the normal development runner.

The test runner:

* Builds the kernel with kernel tests enabled
* Builds the bootloader
* Prepares the EFI boot environment
* Copies the kernel into the test ESP
* Starts QEMU independently
* Captures kernel serial output
* Detects test completion
* Reports the final test result
* Fails on timeout
* Fails when the kernel reports a test failure

This provides a repeatable regression-testing workflow without depending on the interactive development runner.

## Synchronization and Interrupt Safety

Kernel global state is being progressively moved away from unnecessary raw global pointers.

The physical frame allocator is protected by an interrupt-safe spinlock, allowing exception handlers such as the page fault handler to safely access allocator state.

The synchronization layer provides:

* Atomic spinlock acquisition
* RAII-based lock guards
* Interrupt-state preservation
* Interrupt disabling while holding interrupt-sensitive locks
* Automatic interrupt-state restoration

This provides the foundation required for increasingly concurrent kernel subsystems.

## Development Roadmap

### Phase 1 — Kernel Basic Infrastructure

```text
✓ UEFI memory map
✓ Physical frame allocator
✓ Page tables
✓ Virtual memory
✓ Kernel heap
✓ Basic framebuffer output
```

### Phase 2 — Kernel Services

```text
✓ Interrupt and exception foundation
✓ GDT and CPU state
✓ Timer infrastructure
✓ Kernel synchronization
✓ Interrupt-safe locking
✓ Global kernel state audit
```

### Phase 3 — Processes and Userspace

```text
├── Process abstraction
├── Thread abstraction
├── Address spaces
├── Scheduler
├── System calls
└── Userspace runtime
```

### Phase 4 — Filesystem and OS Services

```text
├── VFS architecture
├── Filesystem abstraction
├── Initial filesystem
├── File handles
├── Directories
├── Storage layer
└── Basic OS services
```

### Phase 5 — Python Runtime

```text
├── Python runtime integration
├── Python execution
├── Import system
├── Required kernel interfaces
└── Runtime services
```

### Phase 6 — AI Kernel

```text
├── AI kernel architecture
├── Python-based AI kernel
├── AI runtime layer
├── Memory and context
├── Tools
└── AI services
```

### Phase 7 — Flust

```text
├── Flust/Mentacore integration
├── Python runtime interface
├── Native UI services
├── Required runtime support
├── AI kernel integration
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
* Use serial diagnostics during low-level development.
* Separate development tooling from kernel test infrastructure.
* Document architectural decisions as the system evolves.

The goal is not simply to produce a bootable kernel, but to build a complete operating-system stack whose architecture can eventually support an AI-native development environment.

## Technology

* **Language:** Rust
* **Architecture:** x86_64
* **Boot:** UEFI
* **Kernel:** Custom `no_std` Rust kernel
* **Virtualization / Testing:** QEMU
* **Memory Management:** Physical frame allocator, page tables, demand-paged kernel heap
* **Interrupts:** x86_64 IDT, LAPIC, PIT
* **Synchronization:** Kernel spinlock with interrupt-safe locking
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

The `bootloader` crate is responsible for preparing the machine, loading the kernel, constructing boot information, and transferring control to the kernel.

The `kernel` crate contains the operating-system kernel and its low-level subsystems.

The `scripts` directory contains development and QEMU execution tooling.

## Development Workflow

Mentacore provides separate workflows for development and automated kernel testing.

### Development Runner

The development runner builds the kernel and bootloader, prepares the EFI environment, and starts QEMU for interactive development.

### Test Runner

The test runner builds the kernel with the `kernel-tests` feature and executes the kernel test suite automatically inside QEMU.

This separation keeps normal kernel execution independent from the test infrastructure while still allowing low-level regression testing.

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