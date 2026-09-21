# Boot Protocol ABI

## Purpose

The Boot Protocol ABI defines the binary data contract exchanged between the bootloader and kernel through `BootInfo`.

It guarantees that both components agree on the layout, field types, and meaning of the boot-time information passed across the boot boundary.

## Contract-Defining Files

### `boot_protocol/src/lib.rs`

Defines the shared binary representation of `BootInfo`.

The structure uses `#[repr(C)]`, making its field ordering, field sizes, alignment, and padding part of the ABI.

It also defines the boot protocol version and framebuffer format constants.

### `bootloader/src/handoff.rs`

Completes the bootloader-owned fields that depend on the final UEFI state before transferring control to the kernel.

In particular, it records the final UEFI memory-map address and metadata into `BootInfo`.

### `kernel/src/main.rs`

Defines the kernel-side ABI entry point that receives a pointer to `BootInfo`.

The kernel does not reconstruct the structure; it consumes the representation defined by the shared `boot_protocol` crate.

## `BootInfo` Layout

The current contract is:

```text
+0    version                         u32
+4    padding                         u32

+8    framebuffer_addr                u64
+16   framebuffer_size                u64

+24   framebuffer_width              u32
+28   framebuffer_height             u32
+32   framebuffer_stride             u32
+36   framebuffer_format             u32

+40   memory_map_addr                u64
+48   memory_map_size                u64

+56   memory_map_descriptor_size     u32
+60   memory_map_descriptor_version  u32

+64   kernel_image_addr              u64
+72   kernel_image_size              u64

+80   userspace_image_addr           u64
+88   userspace_image_size           u64
```

The exact layout is determined by the `#[repr(C)]` declaration and is shared by both crates through the `mentacore_boot_protocol` dependency.

## Field Contract

### Protocol

```text
version
```

Identifies the version of the boot protocol used to interpret the structure.

The current protocol version is:

```text
2
```

### Framebuffer

```text
framebuffer_addr
framebuffer_size
framebuffer_width
framebuffer_height
framebuffer_stride
framebuffer_format
```

Describe the framebuffer established by the bootloader and made available to the kernel.

`framebuffer_format` uses the protocol-defined format constants:

```text
FRAMEBUFFER_FORMAT_RGB      = 0
FRAMEBUFFER_FORMAT_BGR      = 1
FRAMEBUFFER_FORMAT_BITMASK  = 2
FRAMEBUFFER_FORMAT_BLT_ONLY = 3
```

### UEFI Memory Map

```text
memory_map_addr
memory_map_size
memory_map_descriptor_size
memory_map_descriptor_version
```

Describe the final UEFI memory map captured when boot services are exited.

The kernel uses these fields as the input for its physical-memory initialization.

### Kernel Image

```text
kernel_image_addr
kernel_image_size
```

Identify the physical memory range containing the kernel image loaded by the bootloader.

### Userspace Image

```text
userspace_image_addr
userspace_image_size
```

Identify the userspace ELF image loaded by the bootloader and made available to the kernel.

## Versioning Contract

`BOOT_PROTOCOL_VERSION` is part of the ABI.

Changes to `BootInfo` must therefore be treated as protocol changes rather than ordinary internal structure changes.

A change to field ordering, field type, field size, alignment-sensitive representation, or field semantics requires corresponding changes on both sides of the boot boundary.

## Critical Representation Conventions

### `#[repr(C)]`

`BootInfo` uses C-compatible representation so that its layout is stable across the bootloader and kernel compilation units.

The ABI depends on the representation, not merely on the Rust source-level field definitions.

### Shared crate

Both components consume `BootInfo` from the same `boot_protocol` crate.

This keeps the binary contract centralized rather than duplicating the structure independently in the bootloader and kernel.

## Boundary Summary

```text
Bootloader
    │
    │ populates BootInfo
    ▼
mentacore_boot_protocol::BootInfo
    │
    │ pointer passed through kernel-entry ABI
    ▼
Kernel
    │
    ├── protocol version
    ├── framebuffer information
    ├── UEFI memory map
    ├── kernel image range
    └── userspace image range
```

The **Boot Protocol ABI** defines the data representation.

The **Kernel Entry / Boot Handoff ABI** defines how execution and the `BootInfo` pointer cross the bootloader/kernel boundary.
