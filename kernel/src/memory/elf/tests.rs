use super::{
    ElfError,
    ParsedElf,
    PF_R,
    PF_X,
    PT_LOAD,
};

const TEST_ELF: &[u8] = &[
    // ELF64 magic / class / endian
    0x7F, b'E', b'L', b'F',
    0x02, 0x01, 0x01, 0x00,

    // padding
    0, 0, 0, 0, 0, 0, 0, 0,

    // e_type = ET_EXEC
    0x02, 0x00,

    // e_machine = x86_64
    0x3E, 0x00,

    // e_version
    0x01, 0x00, 0x00, 0x00,

    // e_entry = 0x0000010000000000
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00,

    // e_phoff = 0x40
    0x40, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,

    // e_shoff = 0
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,

    // e_flags
    0x00, 0x00, 0x00, 0x00,

    // e_ehsize = 64
    0x40, 0x00,

    // e_phentsize = 56
    0x38, 0x00,

    // e_phnum = 1
    0x01, 0x00,

    // e_shentsize = 64
    0x40, 0x00,

    // e_shnum = 13
    0x0D, 0x00,

    // e_shstrndx
    0x0B, 0x00,

    // PT_LOAD
    0x01, 0x00, 0x00, 0x00,

    // PF_R | PF_X
    0x05, 0x00, 0x00, 0x00,

    // p_offset = 0x1000
    0x00, 0x10, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,

    // p_vaddr = 0x0000010000000000
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00,

    // p_paddr
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x01, 0x00, 0x00,

    // p_filesz = 6
    0x06, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,

    // p_memsz = 6
    0x06, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,

    // p_align = 0x1000
    0x00, 0x10, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

pub fn test_valid_elf() -> Result<(), ElfError> {
    let elf = ParsedElf::parse(TEST_ELF)?;

    if elf.entry() != 0x0000_0100_0000_0000 {
        return Err(ElfError::InvalidMagic);
    }

    if elf.segment_count() != 1 {
        return Err(ElfError::InvalidMagic);
    }

    let segment =
        elf.segment(0)
            .ok_or(ElfError::NoLoadSegments)?;

    if segment.offset != 0x1000 {
        return Err(ElfError::InvalidMagic);
    }

    if segment.virtual_address
        != 0x0000_0100_0000_0000
    {
        return Err(ElfError::InvalidMagic);
    }

    if segment.file_size != 6
        || segment.memory_size != 6
    {
        return Err(ElfError::InvalidSegmentSize);
    }

    if !segment.flags.readable
        || !segment.flags.executable
        || segment.flags.writable
    {
        return Err(ElfError::InvalidSegmentSize);
    }

    if PT_LOAD != 1
        || PF_R != 4
        || PF_X != 1
    {
        return Err(ElfError::InvalidSegmentSize);
    }

    Ok(())
}