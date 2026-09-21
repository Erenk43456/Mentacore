# Kernel Entry / Boot Handoff ABI

## Purpose

The Kernel Entry / Boot Handoff ABI defines the contract between the UEFI bootloader and the kernel at the moment control is transferred from the bootloader to `_start`.

It establishes:

* the kernel entry address;
* the kernel stack;
* the `BootInfo` argument;
* the memory layout and representation of `BootInfo`;
* the point at which UEFI boot services have already been exited.

This is the initial machine-level boundary between the bootloader and the kernel.

## Contract-Defining Files

### `bootloader/src/handoff.rs`

Defines the Rust-side handoff interface.

`enter_kernel` receives the kernel ELF entry address and allocated kernel stack top, finalizes the UEFI memory-map information in `BootInfo`, exits UEFI boot services, and invokes the assembly handoff routine.

It declares:

```rust id="o5uk9v"
unsafe extern "C" {
    fn jump_to_kernel(
        entry: u64,
        stack_top: u64,
        boot_info: u64,
    ) -> !;
}
```

### `bootloader/asm/handoff.s`

Defines the final register and stack transition.

The bootloader-side function receives:

```text id="d0d1hc"
RCX = kernel entry address
RDX = kernel stack top
R8  = BootInfo address
```

It then establishes the kernel entry state:

```text id="2cvp7m"
RSP = stack_top
RDI = BootInfo
RIP = kernel entry
```

and transfers control with `jmp`.

The bootloader therefore does not return from the handoff.

### `boot_protocol/src/lib.rs`

Defines the binary representation of the object transferred to the kernel.

`BootInfo` uses `#[repr(C)]`, making its field order and C-compatible layout part of the boot contract.

The current structure contains:

```text id="rj1k9q"
version
framebuffer_addr
framebuffer_size
framebuffer_width
framebuffer_height
framebuffer_stride
framebuffer_format
memory_map_addr
memory_map_size
memory_map_descriptor_size
memory_map_descriptor_version
kernel_image_addr
kernel_image_size
userspace_image_addr
userspace_image_size
```

The boot protocol version is currently `2`.

### `kernel/src/main.rs`

Defines the kernel-side entry point:

```rust id="5eq0an"
#[unsafe(no_mangle)]
pub extern "C" fn _start(
    boot_info: *const BootInfo,
) -> ! {
    kernel_init::start(boot_info)
}
```

The kernel therefore expects the `BootInfo` pointer in `RDI` when `_start` is entered.

## Entry Contract

Immediately before the final jump:

```text id="jgj1u7"
RIP = kernel ELF entry
RSP = allocated kernel stack top
RDI = BootInfo address
```

No return address is placed on the stack.

The transfer is:

```asm id="4o0k5m"
mov rax, rcx
mov rsp, rdx
mov rdi, r8
jmp rax
```

The kernel entry function is therefore entered directly rather than through a normal `call`/`ret` pair.

## BootInfo Contract

`BootInfo` is the shared data structure between the bootloader and kernel.

Its `#[repr(C)]` layout is contract-sensitive: both components must agree on field order, field sizes, and alignment.

The bootloader populates the structure before the handoff, including:

* framebuffer information;
* UEFI memory-map location and metadata;
* kernel image location and size;
* userspace image location and size;
* boot protocol version.

The kernel receives only a pointer to this structure at entry.

## UEFI Boundary

The handoff occurs only after:

```rust
boot::exit_boot_services(None)
```

has completed.

The memory-map fields stored in `BootInfo` therefore describe the final UEFI memory map associated with the transition out of boot services.

After this point, the kernel owns execution and cannot rely on UEFI boot services remaining available.

## Critical Assembly Conventions

### Windows-to-System-V register transition

The bootloader's Rust-to-assembly call uses the Windows x64 calling convention:

```text
RCX = entry
RDX = stack_top
R8  = boot_info
```

`handoff.s` deliberately converts this into the kernel's entry convention:

```text
RDI = BootInfo
```

This is the ABI bridge between the bootloader's Rust environment and the kernel's System V-style entry contract.

### `mov rsp, rdx`

The bootloader-provided kernel stack becomes the active stack before entering `_start`.

The kernel therefore starts execution on the explicitly allocated kernel stack rather than inheriting the UEFI caller's stack.

### `jmp rax`

The transfer uses `jmp` instead of `call`.

There is intentionally no return path: `_start` is a `-> !` entry point and owns execution after the handoff.

## Boundary Summary

```text id="i5f7eu"
UEFI bootloader
    │
    ├── load kernel ELF
    ├── allocate kernel stack
    ├── build BootInfo
    └── exit UEFI boot services
    │
    ▼
jump_to_kernel
    │
    ├── RCX = kernel entry
    ├── RDX = kernel stack
    └── R8  = BootInfo
    │
    ▼
    RSP = stack_top
    RDI = BootInfo
    RIP = kernel entry
    │
    ▼
kernel::_start(*const BootInfo)
```

This ABI is distinct from the **Boot Protocol ABI** at the data-structure level: the handoff ABI defines how execution and the `BootInfo` pointer cross the boundary, while the boot protocol defines the binary contract of the `BootInfo` structure itself.
