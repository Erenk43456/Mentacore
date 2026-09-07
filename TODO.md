# Mentacore — TODO

## Phase 0 — Project Foundation

* [ ] Set up the Rust workspace
* [ ] Define the initial project structure
* [ ] Configure the x86_64 UEFI target
* [ ] Set up a reproducible QEMU development environment
* [ ] Boot a minimal UEFI application
* [ ] Establish serial output for debugging

## Phase 1 — Custom Bootloader

* [ ] Implement the Mentacore bootloader in Rust
* [ ] Load the kernel from the EFI System Partition
* [ ] Parse the kernel image
* [ ] Obtain the UEFI memory map
* [ ] Prepare the initial kernel execution environment
* [ ] Pass boot information to the kernel
* [ ] Transfer execution to the kernel
* [ ] Verify a minimal kernel boot

## Phase 2 — Mentacore Kernel

* [ ] Create the initial Rust kernel
* [ ] Implement kernel entry point
* [ ] Implement basic console output
* [ ] Implement physical memory management
* [ ] Implement virtual memory and paging
* [ ] Implement GDT
* [ ] Implement IDT
* [ ] Implement interrupt handling
* [ ] Implement a kernel heap
* [ ] Implement an allocator
* [ ] Implement basic CPU management
* [ ] Implement process and thread primitives
* [ ] Implement scheduling
* [ ] Implement basic device support
* [ ] Implement filesystem support
* [ ] Establish userspace execution

## Phase 3 — Userspace

* [ ] Define the userspace/kernel interface
* [ ] Implement system calls
* [ ] Create the initial userspace environment
* [ ] Implement a basic process model
* [ ] Create essential userspace services

## Phase 4 — Python Runtime

* [ ] Research suitable Python runtime approaches
* [ ] Define the Python runtime/kernel interface
* [ ] Port or implement a Python runtime suitable for Mentacore
* [ ] Provide the required runtime services
* [ ] Execute Python code directly on Mentacore

## Phase 5 — AI Kernel

* [ ] Define the AI kernel architecture
* [ ] Build the Python-based AI kernel
* [ ] Implement AI-oriented system services
* [ ] Implement model/runtime integration
* [ ] Define the interface between the AI kernel and userspace
* [ ] Establish the AI kernel as the primary high-level environment

## Phase 6 — Flust

* [ ] Define the Flust/Mentacore integration layer
* [ ] Port required Flust components
* [ ] Run Flust on the Mentacore Python environment
* [ ] Integrate Flust with the AI kernel
* [ ] Establish Flust as the primary development environment

## Long-Term Vision

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
Python Runtime
   ↓
AI Kernel
   ↓
Flust
```

## Development Principles

* Build from the lowest layer upward.
* Keep each milestone independently testable.
* Avoid adding complexity before the underlying layer is stable.
* Prefer explicit interfaces between system layers.
* Keep the kernel and boot process understandable and debuggable.
* Document architectural decisions as the project evolves.
