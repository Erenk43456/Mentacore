# UEFI Memory Map / Physical Frame Allocator ABI

## Purpose

The UEFI Memory Map / Physical Frame Allocator ABI defines how physical memory information crosses the bootloader/kernel boundary and becomes allocatable 4 KiB physical frames inside the kernel.

It establishes the binary memory-descriptor format, descriptor iteration rules, and the page-granular contract consumed by the physical frame allocator.

## Contract-Defining Files

### `boot_protocol/src/lib.rs`

Defines the `BootInfo` fields carrying the UEFI memory-map location and metadata:

```text id="2s7v4m"
memory_map_addr
memory_map_size
memory_map_descriptor_size
memory_map_descriptor_version
```

These fields form the bootloader → kernel binary contract for locating and interpreting the memory map.

### `bootloader/src/handoff.rs`

Defines the point at which the final UEFI memory-map metadata is written into `BootInfo`.

After `ExitBootServices`, the bootloader stores:

```text id="9f3k6p"
memory_map.buffer().as_ptr()
memory_map.meta().map_size
memory_map.meta().desc_size
memory_map.meta().desc_version
```

The kernel therefore receives the memory map that was obtained immediately before kernel execution begins.

### `kernel/src/memory/memory_map.rs`

Defines the kernel-side representation and interpretation of each memory descriptor:

```rust id="6v8m2q"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryDescriptor {
    pub ty: u32,
    pub pad: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}
```

The kernel obtains descriptors by applying the descriptor size supplied through `BootInfo`.

The conventional-memory type currently used by the allocator is:

```text id="4x1n8r"
UEFI_CONVENTIONAL_MEMORY = 7
```

### `kernel/src/memory/physical/allocator.rs`

Defines the kernel-side physical-frame allocation contract.

The allocator operates in units of:

```text id="7q5c3w"
PAGE_SIZE = 4096 bytes
```

Its externally meaningful operations are:

```text id="1n9f6b"
allocate_frame()
allocate_frame_below()
allocate_contiguous_frames()
reserve_range()
free_frame()
is_frame_used()
```

Every returned `Frame` represents one page-aligned physical 4 KiB frame.

## Memory Descriptor Contract

The kernel does not assume that descriptors are tightly packed according to the Rust structure size.

Instead, descriptor traversal uses the descriptor size supplied by UEFI:

```text id="8m4t1z"
descriptor_address =
    memory_map_addr
    + index * memory_map_descriptor_size
```

This is part of the ABI because the descriptor stride is supplied by the producer rather than derived from `sizeof(MemoryDescriptor)`.

## Physical Frame Contract

The physical allocator converts memory into fixed-size frames:

```text id="5r2k7v"
physical address
      │
      ├── page-aligned
      │
      ▼
Frame {
    start_address
}
```

A valid frame therefore satisfies:

```text id="0x6q8m"
start_address % 4096 == 0
```

The frame address is the physical address subsequently consumed by the paging subsystem.

## Conventional Memory Contract

Only UEFI descriptors whose type is `UEFI_CONVENTIONAL_MEMORY` are considered available for general physical-memory allocation.

Reserved, firmware, runtime, or otherwise non-conventional regions are not converted into allocatable frames by this interface.

The memory-map layer also provides the physical range information used when locating allocator metadata.

## Allocator Boundary

The allocator's bitmap tracks physical frames rather than arbitrary byte ranges.

Conceptually:

```text id="3h7p2n"
UEFI memory descriptors
        │
        ▼
conventional physical ranges
        │
        ▼
4 KiB frame numbers
        │
        ▼
FrameBitmap
        │
        ▼
PhysicalFrameAllocator
        │
        ▼
Frame { start_address }
```

The bitmap therefore represents the ownership state of physical page frames, while `MemoryMap` represents the original UEFI memory description.

## Low-Memory Constraint

The memory-map implementation reserves a bounded region for allocator metadata.

The current search policy avoids physical address zero and keeps the selected bitmap region below:

```text id="9c5v1x"
0x1_0000_0000
```

This is an implementation constraint required by the current identity-mapped low-memory environment; it is not a general UEFI memory-map rule.

## Boundary Summary

```text id="7n4m8q"
UEFI
 │
 │ final memory map
 ▼
BootInfo
 │
 │ address + size + descriptor stride
 ▼
MemoryMap
 │
 │ conventional-memory descriptors
 ▼
PhysicalFrameAllocator
 │
 │ 4 KiB physical frames
 ▼
Frame
 │
 ▼
Paging / AddressSpace
```

The **UEFI Memory Map / Physical Frame Allocator ABI** defines the physical-memory boundary from firmware-provided memory descriptors to kernel-owned page frames. It does not define virtual-memory mappings, page-table entry formats, or address-space switching.
