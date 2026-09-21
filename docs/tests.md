# Mentacore Test Infrastructure

Mentacore uses a custom kernel-side testing infrastructure designed for its `no_std` execution environment.

Because the kernel does not run under the standard Rust test harness, tests are compiled into the kernel test configuration and executed directly inside QEMU. Results are reported through the serial console and evaluated by the project's regression test runner.

The test infrastructure is intended to verify both isolated kernel subsystems and the integration between them.

---

## Test Architecture

The test system consists of three layers:

```text
┌─────────────────────────────────────────────┐
│              Kernel Test Suites             │
│                                             │
│  memory / paging / ELF / process / thread  │
│  scheduler / syscall / userspace / heap     │
│  CPU / interrupts / synchronization / traps │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│             Kernel Test Harness             │
│                                             │
│  test registration                          │
│  test execution                             │
│  serial reporting                           │
│  aggregate result tracking                  │
└──────────────────────┬──────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────┐
│             QEMU Regression Runner          │
│                                             │
│  build → EFI preparation → QEMU → serial   │
│  output → result detection → timeout        │
└─────────────────────────────────────────────┘
```

This separation allows individual kernel tests to remain relatively independent while the QEMU runner validates the complete boot and execution path.

---

## Test Configuration

Kernel tests are enabled through the `kernel-tests` Cargo feature.

The test configuration is intentionally separate from the normal kernel build. This allows the production kernel configuration and the regression configuration to remain distinct while sharing the same kernel implementation.

The test kernel is still a real kernel image. Tests execute in the same `no_std` environment rather than being translated into host-side unit tests.

This is important for subsystems whose behavior depends on:

* page tables
* physical frame allocation
* CR3/address-space changes
* interrupt delivery
* CPU state
* kernel stacks
* privilege-level transitions
* scheduler state
* userspace execution
* hardware-visible memory mappings

---

# Test Suites

The kernel test infrastructure is organized by subsystem. Each test has a stable identifier in the form:

```text
<subsystem>::<test_name>
```

The identifiers below correspond to the kernel regression suite and document the behavior currently covered by the test infrastructure.

---

## Physical Memory

Physical memory tests validate frame allocation, reuse, reservation, validation, and physical-memory boundary handling.

```text
physical::frame_free
physical::frame_reuse
physical::counters
physical::double_free
physical::invalid_frames
physical::contiguous_allocation
physical::reservation
physical::invalid_reservation
physical::conventional_region_below_4g
```

### Coverage

These tests cover:

* individual frame allocation and release
* frame reuse
* allocator counters
* invalid and duplicate frame operations
* contiguous physical-frame allocation
* frame reservation
* invalid reservation handling
* conventional-memory regions below the 4 GiB boundary

---

## Paging and Address Spaces

Paging tests validate page-table operations, mapping semantics, address-space creation, and physical/virtual address relationships.

```text
paging::duplicate_mapping
paging::mapping
paging::unmap
paging::unmap_rejection
paging::user_mapping_permissions
paging::virtual_address_layout
paging::address_space
paging::address_space_mapping
paging::address_space_unmapping
paging::nx_page_flags
paging::physical_to_virtual_mapping
paging::address_space_drop
```

### Coverage

These tests cover:

* duplicate mapping rejection
* virtual-to-physical mapping
* page unmapping
* invalid unmapping
* user-access permissions
* virtual address layout
* address-space creation
* address-space-specific mappings
* address-space-specific unmapping
* NX page permissions
* physical-to-virtual address conversion
* address-space cleanup and destruction

Paging tests form an important integration layer between the physical frame allocator and higher-level process and userspace functionality.

---

## ELF Loading

ELF tests validate parsing and loading of userspace executable images.

```text
elf::valid_header
elf::load_segment
elf::segment_data
elf::invalid_magic
elf::truncated_header
elf::program_headers_out_of_bounds
elf::invalid_segment_size
elf::invalid_alignment
elf::no_load_segments
elf::entry_not_in_load_segment
elf::file_range_out_of_bounds
elf::kernel_space_segment_rejected
elf::loader_entry
elf::loader_segment
elf::loader_overlapping_segments
```

### Coverage

These tests cover:

* valid ELF headers
* loadable segment processing
* segment data extraction
* invalid ELF magic
* truncated headers
* invalid program-header ranges
* invalid segment sizes
* invalid alignment
* ELF files without loadable segments
* invalid entry points
* file-range validation
* rejection of kernel-space segments
* loader entry-point setup
* segment loading
* overlapping segment detection

The ELF suite validates both malformed input rejection and valid userspace image construction.

---

## Processes and Threads

Process tests validate process creation and state. Thread tests validate execution contexts, startup state, kernel stacks, and thread management.

### Process Tests

```text
process::creation
process::state
```

### Thread Tests

```text
thread::creation
thread::state
thread::context
thread::interrupt_context
thread::user_interrupt_context
thread::kernel_context_layout
thread::context_switch
thread::context_startup
thread::kernel_stack
thread::kernel_stack_direct
thread::kernel_stack_validation
thread::manager_creation
thread::manager_operations
thread::kernel_stack_allocation
thread::kernel_stack_drop
```

### Coverage

Process tests cover:

* process creation
* process state

Thread tests cover:

* thread creation
* thread state transitions
* saved kernel context
* interrupt context construction
* userspace interrupt context construction
* kernel-context memory layout
* context-switch preparation
* initial thread startup state
* kernel stack creation
* direct kernel-stack access
* kernel-stack validation
* thread-manager initialization
* thread-manager operations
* kernel-stack allocation
* kernel-stack cleanup

The context and interrupt-context tests are particularly important because scheduler and userspace execution depend on exact low-level frame layouts.

---

## Scheduler

Scheduler tests validate runnable-thread management, queue behavior, scheduling policy, managed-thread lookup, runtime state, and preemption.

```text
scheduler::creation
scheduler::queue_creation
scheduler::queue_push
scheduler::queue_duplicate
scheduler::queue_remove
scheduler::queue_missing_remove
scheduler::round_robin_selection
scheduler::round_robin_remove_current
scheduler::round_robin_empty
scheduler::managed_thread
scheduler::unknown_thread
scheduler::managed_thread_selection
scheduler::thread_states
scheduler::context_switch
scheduler::runtime
scheduler::preemption
scheduler::timer_preemption
```

### Coverage

These tests cover:

* scheduler initialization
* run-queue creation
* thread insertion
* duplicate insertion handling
* thread removal
* missing-thread removal
* round-robin selection
* removal of the currently selected thread
* empty queue behavior
* managed-thread lookup
* unknown-thread handling
* scheduler-managed thread selection
* scheduler/thread-state interaction
* context-switch preparation
* scheduler runtime initialization
* preemption state
* timer-driven preemption

Scheduler tests exercise the boundary between high-level scheduling decisions and low-level context-switch machinery.

---

## System Calls and Userspace

System-call tests validate the syscall register contract, dispatch logic, and user-memory validation.

```text
syscall::context_registers
syscall::dispatch_get_tid
syscall::dispatch_unknown
syscall::user_address_validation
syscall::user_buffer_validation
```

### Coverage

These tests cover:

* syscall context register layout
* valid syscall dispatch
* unknown syscall handling
* userspace address validation
* userspace buffer validation

Userspace execution is also exercised indirectly by the interrupt, scheduler, process, thread, and syscall integration tests.

Where applicable, the userspace test path validates:

```text
userspace
   │
   ▼
int 0x80
   │
   ▼
IDT syscall gate
   │
   ▼
syscall entry
   │
   ▼
SyscallContext
   │
   ▼
dispatcher
   │
   ▼
return value
```

---

## Heap

Heap tests validate kernel dynamic allocation and multi-page allocation behavior.

```text
heap::basic_allocation
heap::multi_page_allocation
```

### Coverage

These tests cover:

* basic kernel heap allocation
* allocations spanning multiple pages

Heap behavior depends on the underlying virtual-memory mapping infrastructure, making these tests an additional integration point between the heap and paging subsystems.

---

## CPU

CPU tests validate processor-related initialization and CPU-specific runtime functionality.

```text
cpu::tsc_calibration
cpu::user_gdt_segments
```

### Coverage

These tests cover:

* TSC calibration
* user/kernel GDT segment configuration

The GDT test is particularly relevant to userspace transitions because privilege-level execution depends on the correct segment descriptors and selectors.

---

## Interrupts

Interrupt tests validate interrupt state, frame construction, timer entry behavior, stack alignment, timer stability, and double-fault handling.

```text
interrupts::state
interrupts::TrapFrame
interrupts::timer_stack_alignment
interrupts::timer_stability
interrupts::lapic_timer_stack_alignment
interrupts::lapic_timer_periodic
interrupts::timer_context
interrupts::timer_preemption
interrupts::double_fault_ist1
interrupts::ring3_transition
```

### Coverage

These tests cover:

* interrupt enable/disable state
* `TrapFrame` layout
* timer interrupt stack alignment
* timer stability
* LAPIC timer stack alignment
* periodic LAPIC timer behavior
* timer interrupt context construction
* timer-driven scheduler preemption
* double-fault IST1 configuration
* ring-3 transition behavior

Interrupt tests are closely coupled to the exception, timer, scheduler, GDT, and TSS contracts.

---

## Synchronization

Synchronization tests validate the kernel's spinlock implementation and interrupt-safe locking behavior.

```text
sync::spinlock
sync::spinlock_irqsave
```

### Coverage

These tests cover:

* normal spinlock acquisition/release
* interrupt-state-preserving lock acquisition
* interrupt-state restoration after lock release

These primitives are used by shared kernel runtime state, including scheduler and memory-management infrastructure.

---

## Exception Handling

Exception handling is exercised primarily through the CPU/interrupt infrastructure and exception-entry paths.

The current regression suite specifically validates double-fault handling through:

```text
interrupts::double_fault_ist1
```

This verifies the dedicated IST1 path used by the double-fault handler.

Exception-entry correctness is additionally exercised by the broader interrupt, timer, userspace, and context-switch tests because these paths depend on compatible CPU-generated and software-generated stack frames.

---

## Test Identifier Stability

Test identifiers serve two purposes:

1. They identify the behavior being validated.
2. They make serial regression output directly traceable to a source-level test.

For example:

```text
[PASS] paging::address_space
[PASS] thread::context_switch
[PASS] scheduler::round_robin_selection
[PASS] syscall::dispatch_get_tid
```

When adding a new test, the identifier should describe the behavior being validated rather than the implementation detail of the test itself.

A test should normally remain in the suite corresponding to the subsystem whose contract it validates.

---

## Suite-Level Integration

The suites are not completely independent.

Important integration relationships include:

```text
Physical Memory
       │
       ▼
    Paging
       │
       ├──────────────► Heap
       │
       ▼
Address Spaces
       │
       ▼
   ELF Loader
       │
       ▼
    Process
       │
       ▼
    Thread
       │
       ▼
  Scheduler
       │
       ├──────────────► Timer / Interrupts
       │
       ▼
 Context Switch
       │
       ▼
   Userspace
       │
       ▼
   Syscalls
```

Interrupts, GDT/TSS state, and synchronization infrastructure support multiple layers of this execution path.

Therefore, a passing subsystem test does not replace the QEMU regression run. The complete regression suite verifies that these individually tested components continue to operate together inside the kernel.

---

# Test Execution Model

Kernel tests do not execute as ordinary host processes.

The execution path is:

```text
Cargo
  │
  ├── Build kernel with `kernel-tests`
  │
  ▼
Kernel Test Image
  │
  ▼
UEFI Bootloader
  │
  ▼
QEMU
  │
  ▼
Mentacore Kernel
  │
  ▼
Kernel Test Harness
  │
  ├── Run test suites
  ├── Report individual results
  └── Emit aggregate result
  │
  ▼
Serial Console
  │
  ▼
QEMU Regression Runner
```

This means a passing test represents successful execution inside the actual kernel environment rather than merely successful execution on the development host.

---

# Serial Test Reporting

The kernel test infrastructure uses serial output as its primary machine-readable reporting channel.

The test harness reports individual test execution and aggregate results through the serial console.

The regression runner uses this output to determine whether the test run completed successfully.

A successful run terminates with:

```text
ALL TESTS PASSED
```

The runner treats this aggregate marker as the completion signal for the complete regression suite.

This avoids relying solely on QEMU process termination, which would not by itself distinguish between:

* successful test completion
* kernel panic
* triple fault
* boot failure
* early initialization failure
* timeout
* unexpected QEMU termination

---

# Test Result Model

Tests are evaluated at both the individual and aggregate levels.

A typical test run conceptually follows:

```text
TEST START
    │
    ▼
Execute test
    │
    ├── PASS ──► Record success
    │
    └── FAIL ──► Record failure
                    │
                    ▼
              Stop/abort as configured
    │
    ▼
Aggregate results
    │
    ▼
ALL TESTS PASSED
```

The aggregate result is important because a kernel can successfully boot and execute many tests while still failing one subsystem-specific test.

The regression runner therefore requires the complete test suite to report successful completion.

---

# QEMU Regression Runner

The project provides a dedicated QEMU regression runner through:

```text
scripts/test.ps1
```

The runner provides a repeatable environment for executing the complete kernel test configuration.

The general workflow is:

```text
1. Build kernel tests
2. Build bootloader
3. Prepare EFI test environment
4. Place kernel image into test ESP
5. Start QEMU
6. Capture serial output
7. Monitor test progress
8. Detect individual failures
9. Detect aggregate completion
10. Enforce timeout
11. Validate final regression result
```

The exact implementation of the runner may evolve, but these responsibilities define its role in the development workflow.

---

## 1. Build Kernel Tests

The runner first builds the kernel with the `kernel-tests` feature enabled.

This ensures that the image being executed contains the current test suites.

A successful Rust compilation is not considered a successful regression run by itself.

Compilation verifies source-level correctness; QEMU execution verifies runtime behavior.

---

## 2. Build Bootloader

The bootloader is built for the test configuration.

This ensures that the kernel image is executed through the same boot path used by the project's UEFI environment.

---

## 3. Prepare EFI Environment

The runner prepares the test EFI system partition and required firmware environment.

The test environment contains the bootloader and kernel image required to start Mentacore under QEMU.

---

## 4. Launch QEMU

QEMU provides the hardware execution environment for the kernel.

The test environment allows the kernel to exercise real x86_64 mechanisms including:

* CPU initialization
* page tables
* interrupts
* timer delivery
* privilege transitions
* CR3 changes
* kernel/user execution
* serial I/O

The regression runner does not replace these mechanisms with host-side mocks.

---

## 5. Capture Serial Output

Serial output is captured by the runner and used as the primary test result stream.

This provides a deterministic channel for communicating kernel test results back to the host-side test runner.

---

## 6. Detect Failures

The runner monitors serial output for failure conditions.

This allows failures to be detected even when the kernel does not exit normally.

Examples include:

* explicit test failure
* kernel panic
* unexpected termination
* missing completion marker
* timeout

---

## 7. Detect Successful Completion

The runner waits for:

```text
ALL TESTS PASSED
```

This marker indicates that the kernel-side test harness completed its registered test suite successfully.

The QEMU process continuing to run after this point does not invalidate the test result; the aggregate completion marker is the authoritative test signal.

---

## 8. Timeout Handling

The regression runner enforces a timeout.

A timeout is treated as a test failure because it indicates that the kernel did not reach a known terminal test state within the expected execution window.

This is particularly useful for detecting failures such as:

* deadlocks
* scheduler stalls
* interrupt failures
* infinite loops
* userspace transition hangs
* silent kernel faults
* QEMU execution stalls

A timeout therefore provides information that a simple process-exit check would not provide.

---

# Development Runner vs. Regression Runner

Mentacore has two different execution purposes.

## Development Runner

The normal QEMU development runner is intended for interactive kernel development.

It is useful for:

* manually booting Mentacore
* observing serial output
* debugging kernel behavior
* testing development builds
* inspecting runtime state

It is not the authoritative regression mechanism.

## Test Runner

The test runner is designed for repeatable validation.

It:

* builds the test configuration
* creates the test boot environment
* launches QEMU
* captures serial output
* detects failures
* waits for aggregate completion
* enforces timeouts
* reports the regression result

This distinction keeps interactive development separate from automated validation.

---

# Current Regression Status

The current kernel test suite reports:

```text
95/95 TESTS PASSED

ALL TESTS PASSED
```

This indicates that all currently registered kernel tests completed successfully in the QEMU regression environment.

The number of tests is expected to change as new kernel subsystems and regression cases are added.

The aggregate count should therefore be treated as a snapshot of the current test suite rather than a permanent project invariant.

---

# What the Regression Suite Validates

The regression suite is intentionally broader than isolated unit testing.

A successful QEMU run simultaneously validates several layers:

```text
Source Code
    │
    ▼
Rust Compilation
    │
    ▼
Kernel Image
    │
    ▼
Bootloader
    │
    ▼
UEFI Boot Path
    │
    ▼
CPU Initialization
    │
    ▼
Memory / Paging
    │
    ▼
Interrupt Infrastructure
    │
    ▼
Kernel Test Harness
    │
    ▼
Subsystem Tests
    │
    ▼
Userspace / Scheduler Integration
    │
    ▼
Aggregate Test Result
```

Consequently, a regression can occur before an individual subsystem test is reached.

For example, an incorrect GDT or page-table configuration can prevent the test harness from starting at all. The absence of `ALL TESTS PASSED` therefore also carries diagnostic information.

---

# Regression Policy

Kernel changes are not considered complete solely because they compile.

For changes that introduce or modify kernel behavior, the expected validation sequence is:

```text
cargo fmt --all
    │
    ▼
cargo check
    │
    ▼
.\scripts\test.ps1
```

Subsystem-specific tests should be added or updated when a new kernel feature introduces behavior that can be tested deterministically.

The QEMU regression runner should then be executed to verify that the change does not break existing kernel behavior.

This is particularly important for changes involving:

* memory management
* paging
* address spaces
* interrupts
* context switching
* scheduling
* process/thread startup
* syscalls
* userspace execution
* synchronization
* kernel initialization

---

# Adding Tests

New tests should be placed with the subsystem they validate rather than collected into a single unrelated test module.

A subsystem test should ideally verify one meaningful behavior or invariant.

Good kernel tests should:

* have deterministic outcomes
* avoid unnecessary timing assumptions
* validate observable behavior or invariants
* isolate the failure as much as practical
* use the kernel test harness rather than host-side assumptions
* remain compatible with the `no_std` execution environment

When a bug is fixed, a regression test should be added when the failure can be reproduced deterministically.

This prevents the same class of failure from silently returning during later kernel development.

---

# Test Coverage Philosophy

Mentacore's test infrastructure is intended to provide multiple levels of confidence:

### Unit-Level Behavior

Small tests validate individual kernel primitives and data structures.

### Subsystem Behavior

Tests validate interactions within a subsystem, such as paging with physical frame allocation.

### Integration Behavior

Tests validate interactions between major subsystems, such as:

```text
Process
   │
   ├── Address Space
   │       │
   │       └── Page Tables
   │
   ├── Thread
   │       │
   │       └── Kernel Context
   │
   └── Scheduler
           │
           └── Context Switch
```

### System-Level Regression

The complete kernel executes inside QEMU and must reach the aggregate success state.

This layered approach makes it possible to distinguish a localized subsystem failure from a failure that prevents the kernel from reaching the test harness at all.

---

# Test Environment

The regression environment can be summarized as:

```text
Host Windows
    │
    ├── Cargo / Rust
    │
    ├── PowerShell
    │
    └── scripts/test.ps1
             │
             ▼
          QEMU
             │
             ▼
       UEFI Firmware
             │
             ▼
        Bootloader
             │
             ▼
        Mentacore Kernel
             │
             ▼
       Kernel Test Harness
             │
             ▼
       Serial Test Output
             │
             ▼
       Regression Result
```

The host-side runner controls the environment, but the tests themselves execute inside the Mentacore kernel.

---

# Summary

Mentacore's test infrastructure is designed around one principle:

> Kernel behavior must be validated in the environment in which the kernel actually runs.

The project therefore combines kernel-side tests with QEMU-based regression execution.

The kernel test suites validate individual subsystems and their interactions, while the QEMU runner verifies that the complete system can boot, execute the test harness, complete the registered tests, and report a successful aggregate result.

The current regression baseline is:

```text
95/95 TESTS PASSED
```

Future kernel features should extend this infrastructure with appropriate tests and preserve the existing QEMU regression baseline.
