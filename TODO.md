# Mentacore — TODO

## Phase 1 — Kernel Basic Infrastructure

### 1.1 Boot and Kernel Foundation

* [x] Rust workspace
* [x] x86_64 UEFI target
* [x] Reproducible QEMU development environment
* [x] Custom Rust UEFI bootloader
* [x] Kernel loading
* [x] UEFI memory map acquisition
* [x] Boot protocol
* [x] Kernel entry point
* [x] Serial debugging
* [x] Framebuffer output

### 1.2 Physical Memory

* [x] Parse UEFI memory map
* [x] Identify conventional memory
* [x] Calculate physical frame count
* [x] Implement physical frame bitmap
* [x] Implement frame allocation
* [x] Implement frame freeing
* [x] Detect invalid frames
* [x] Detect double-free
* [x] Track live allocations
* [x] Track total allocations

### 1.3 Virtual Memory

* [x] Create initial page tables
* [x] Create identity mapping
* [x] Implement 2 MiB huge-page mapping
* [x] Implement 4 KiB page mapping
* [x] Implement page unmapping
* [x] Validate canonical virtual addresses
* [x] Validate physical addresses
* [x] Detect duplicate mappings
* [x] Reject invalid unmapping
* [x] Load kernel page tables into CR3
* [x] Implement `Mapper` abstraction

### 1.4 Kernel Heap

* [x] Define kernel heap virtual address range
* [x] Implement global allocator
* [x] Implement bump allocation
* [x] Demand-page heap memory
* [x] Connect heap page faults to physical frame allocation
* [x] Map newly allocated heap pages
* [x] Invalidate TLB after page mapping
* [x] Validate single-page allocations
* [x] Validate multi-page allocations

### 1.5 Phase 1 Validation

* [x] Physical frame free test
* [x] Physical frame reuse test
* [x] Physical frame counter test
* [x] Physical frame double-free test
* [x] Physical frame invalid-frame test
* [x] Paging duplicate-map test
* [x] Paging map test
* [x] Paging unmap test
* [x] Paging invalid-unmap test
* [x] Heap allocation test
* [x] Multi-page heap allocation test
* [x] Display test
* [x] QEMU boot validation
* [x] Zero-warning build

---

# Phase 2 — Kernel Services

## 2.1 Interrupts and Exceptions

### IDT Foundation

* [x] Define 256-entry IDT
* [x] Define IDT entry structure
* [x] Define IDTR structure
* [x] Implement IDT handler registration
* [x] Load IDT with `lidt`
* [x] Read kernel code segment
* [x] Install page fault handler

### Exception Infrastructure

* [x] Divide Error (#DE)
* [x] Invalid Opcode (#UD)
* [x] General Protection Fault (#GP)
* [x] Page Fault (#PF)
* [ ] Common exception reporting infrastructure
* [ ] Register dump helper
* [ ] Standardized exception diagnostics
* [ ] Double Fault (#DF)
* [ ] Stack Segment Fault (#SS)
* [ ] Invalid TSS (#TS)
* [ ] Segment Not Present (#NP)
* [ ] Alignment Check (#AC)
* [ ] Machine Check (#MC)

### Page Fault Infrastructure

* [x] Page fault entry stub
* [x] Preserve general-purpose registers
* [x] Read CPU error code
* [x] Read CR2
* [x] Capture faulting RIP
* [x] Capture CS
* [x] Capture RFLAGS
* [x] Detect protection faults
* [x] Detect kernel heap faults
* [x] Allocate physical frame on demand
* [x] Map faulting heap page
* [x] Invalidate TLB
* [x] Resume execution with `iretq`

### Interrupts

* [ ] Define hardware interrupt architecture
* [ ] Configure interrupt controller
* [ ] Implement interrupt entry stubs
* [ ] Implement interrupt dispatch
* [ ] Implement interrupt registration
* [ ] Implement interrupt masking
* [ ] Implement interrupt acknowledgement

---

## 2.2 GDT and CPU State

* [ ] Define GDT
* [ ] Define code segment
* [ ] Define data segment
* [ ] Load GDT with `lgdt`
* [ ] Implement segment reload
* [ ] Define Task State Segment
* [ ] Load TSS
* [ ] Configure kernel stack handling
* [ ] Define CPU initialization layer
* [ ] Add CPU feature detection

---

## 2.3 Timer

* [ ] Select initial timer source
* [ ] Initialize timer hardware
* [ ] Configure timer frequency
* [ ] Implement timer interrupts
* [ ] Track system ticks
* [ ] Implement monotonic time source
* [ ] Validate timer stability

---

## 2.4 Kernel Synchronization

* [x] Define interrupt-safe synchronization primitives
* [x] Implement spinlock
* [x] Implement interrupt-safe locking
* [x] Audit global mutable kernel state
* [x] Remove unnecessary raw global pointers where possible

---

# Phase 3 — Processes and Userspace

## 3.1 Process Infrastructure

* [ ] Define process abstraction
* [ ] Define thread abstraction
* [ ] Define process states
* [ ] Define kernel stack per thread
* [ ] Implement context representation
* [ ] Implement context switching
* [ ] Implement process address spaces
* [ ] Implement user page tables

## 3.2 Scheduler

* [ ] Define scheduler architecture
* [ ] Implement runnable queue
* [ ] Implement basic scheduler
* [ ] Implement timer-driven scheduling
* [ ] Implement context switching
* [ ] Implement idle thread

## 3.3 System Calls

* [ ] Define syscall ABI
* [ ] Implement syscall entry
* [ ] Implement syscall dispatch
* [ ] Define process-related syscalls
* [ ] Define memory-related syscalls
* [ ] Define filesystem-related syscalls
* [ ] Validate user/kernel boundary

## 3.4 Userspace

* [ ] Create first userspace address space
* [ ] Load a userspace executable
* [ ] Start first userspace process
* [ ] Implement basic userspace runtime
* [ ] Implement userspace services

---

# Phase 4 — Filesystem and OS Services

* [ ] Define VFS architecture
* [ ] Implement filesystem abstraction
* [ ] Implement initial filesystem
* [ ] Implement file handles
* [ ] Implement directories
* [ ] Implement basic storage layer
* [ ] Implement process filesystem interface
* [ ] Implement basic OS services

---

# Phase 5 — Python Runtime

* [ ] Research suitable Python runtime
* [ ] Evaluate RustPython integration
* [ ] Define Python/kernel interface
* [ ] Define Python/userspace interface
* [ ] Integrate Python runtime
* [ ] Implement required memory services
* [ ] Implement required filesystem services
* [ ] Implement required process services
* [ ] Implement import system
* [ ] Implement required standard-library support
* [ ] Execute Python code directly on Mentacore
* [ ] Validate long-running Python processes

---

# Phase 6 — AI Kernel

* [ ] Define AI kernel architecture
* [ ] Define AI kernel/runtime interface
* [ ] Build Python-based AI kernel
* [ ] Implement AI-oriented system services
* [ ] Implement model/runtime integration
* [ ] Implement AI memory/context layer
* [ ] Implement tool system
* [ ] Define AI kernel/userspace interface
* [ ] Establish AI kernel as the primary high-level environment

---

# Phase 7 — Flust

* [ ] Define Flust/Mentacore integration layer
* [ ] Define Flust/Python runtime interface
* [ ] Port required Flust components
* [ ] Replace unsupported platform dependencies
* [ ] Implement required native UI services
* [ ] Run Flust on Mentacore Python runtime
* [ ] Integrate Flust with the AI kernel
* [ ] Establish Flust as the primary development environment

---

# Long-Term Architecture

```text
Hardware
   ↓
UEFI
   ↓
Mentacore Bootloader
   ↓
Mentacore Kernel
   ↓
Userspace
   ↓
Filesystem / OS Services
   ↓
Python Runtime
   ↓
AI Kernel
   ↓
Flust
```

# Development Principles

* [x] Build from the lowest layer upward.
* [x] Keep milestones independently testable.
* [x] Validate infrastructure before building on top of it.
* [x] Prefer explicit interfaces between layers.
* [x] Keep low-level code understandable and debuggable.
* [x] Use serial diagnostics during kernel development.
* [ ] Document important architectural decisions as the project evolves.
* [ ] Maintain regression tests for every completed subsystem.
* [ ] Avoid introducing higher-level dependencies before the required kernel infrastructure exists.
