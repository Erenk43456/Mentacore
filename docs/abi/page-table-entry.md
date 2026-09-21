# Page-Table Entry / Physical Frame ABI

## Purpose

The Page-Table Entry / Physical Frame ABI defines the binary contract between Mentacore's Rust paging structures and the x86_64 page-table format consumed by the CPU.

It establishes page-table dimensions, physical-address encoding, page-table index calculation, and the architectural flag bits used in page-table entries.

## Contract-Defining Files

### `kernel/src/memory/paging/mod.rs`

Defines the architectural page-table representation and shared entry constants.

The page size is:

```text id="4q6j1m"
PAGE_SIZE = 4096
```

Each page table contains 512 entries:

```text id="8m2k7r"
ENTRY_COUNT = 512
```

The physical-address portion of an entry is extracted with:

```text id="1f5x8p"
ADDRESS_MASK = 0x000f_ffff_ffff_f000
```

The CPU-visible entry flags currently defined by the kernel are:

```text id="6c9v3s"
PRESENT = 1 << 0
WRITABLE = 1 << 1
USER = 1 << 2
PCD = 1 << 4
HUGE_PAGE = 1 << 7
OWNED = 1 << 9
NX = 1 << 63
```

`OWNED` is a kernel bookkeeping bit; it is not an architectural page-access permission.

### `kernel/src/memory/paging/table.rs`

Defines the page-table hierarchy and virtual-address index calculation.

A canonical virtual address is decomposed into:

```text id="p8v3n2"
PML4 index = (address >> 39) & 0x1ff
PDPT index = (address >> 30) & 0x1ff
PD index   = (address >> 21) & 0x1ff
PT index   = (address >> 12) & 0x1ff
```

Each level therefore consumes 9 address bits, followed by the 12-bit page offset.

The hierarchy is:

```text id="r7k4d1"
PML4
 └── PDPT
      └── PD
           └── PT
                └── 4 KiB page
```

The file also defines the conversion from a physical page-table address to the virtual address used by the kernel to access that table.

### `kernel/src/memory/paging/mapper.rs`

Defines how `PageFlags` are converted into CPU page-table entry bits.

The mapping is:

```text id="m5t8q0"
writable       → WRITABLE
cache_disable  → PCD
user           → USER
!executable    → NX
```

Every normal mapped page also receives `PRESENT`.

The resulting PTE has the form:

```text id="v2c6y9"
physical page address
        |
        + architectural flags
        |
        + kernel ownership metadata where applicable
```

The physical address is required to be page-aligned and within the supported physical-address range.

### `kernel/src/memory/physical/frame.rs`

Defines the Rust representation of a physical page frame:

```rust id="9w3h6k"
#[derive(Clone, Copy)]
pub struct Frame {
    pub start_address: u64,
}
```

`Frame::new()` accepts only page-aligned physical addresses.

Therefore, a `Frame` represents the same 4 KiB physical-page granularity used by the page-table format.

## Page-Table Entry Contract

A normal 4 KiB page-table entry stores:

```text id="5n7q2x"
63                         12 11       0
+---------------------------+-----------+
|     physical page addr    |   flags   |
+---------------------------+-----------+
```

The physical page address is extracted with `ADDRESS_MASK`.

The low bits contain architectural attributes such as `PRESENT`, `WRITABLE`, and `USER`, while bit 63 controls execute permission through `NX`.

## Physical Frame Contract

A physical frame address used by the paging subsystem must satisfy:

```text id="3p8m4v"
address % PAGE_SIZE == 0
```

The frame address is placed directly into the address portion of a page-table entry.

Consequently:

```text id="w6j2r9"
Frame.start_address
        │
        ▼
PTE physical-address field
        │
        ▼
CPU physical page
```

## Large-Page Contract

The paging implementation also uses 2 MiB pages:

```text id="0q5s8n"
HUGE_PAGE_SIZE = 0x20_0000
```

A page-directory entry with `HUGE_PAGE` set represents a large page rather than pointing to a lower-level page table.

This distinction is part of the page-table interpretation contract:

```text
PD entry
 ├── HUGE_PAGE clear → points to PT
 └── HUGE_PAGE set   → maps a 2 MiB page
```

## Physical Table Access

Page-table entries contain physical addresses, while the Rust implementation needs a virtual pointer to access the corresponding table.

`physical_table_pointer()` therefore applies the kernel's physical-memory mapping when the physical address is outside the identity-mapped region.

The resulting relationship is:

```text id="c4h7n1"
physical table address
        │
        ├── below identity-map limit
        │       └── physical address used directly
        │
        └── otherwise
                └── PHYS_MAP_BASE + physical address
```

This conversion is an implementation boundary around the architectural physical address stored in the entry.

## Critical Low-Level Convention

### Address and flags must remain separable

When a page-table entry is read, its physical address must be obtained through:

```text
entry & ADDRESS_MASK
```

rather than treating the complete entry value as an address.

The lower flag bits and `NX` occupy the same 64-bit value as the physical page address.

### Page alignment is part of the contract

Both `Frame` creation and normal page mapping require page-aligned addresses.

A non-aligned physical address cannot be represented as the base address of a normal 4 KiB page-table mapping.

## Boundary Summary

```text id="u1p5k8"
Physical Frame
      │
      │ aligned physical address
      ▼
PageTable entry
      │
      ├── physical address bits
      └── architectural flags
              │
              ▼
             CPU
              │
              ▼
       virtual → physical
```

The **Page-Table Entry / Physical Frame ABI** defines the binary representation consumed by the MMU. It does not define higher-level address-space ownership, mapping policy, heap layout, or process isolation policy.
