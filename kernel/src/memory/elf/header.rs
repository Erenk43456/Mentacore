use super::ElfError;

pub const ELF_HEADER_SIZE: usize = 64;
pub const ELF_CLASS_64: u8 = 2;
pub const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
pub const ELF_TYPE_EXEC: u16 = 2;
pub const ELF_MACHINE_X86_64: u16 = 0x3E;

#[derive(Clone, Copy)]
pub struct ElfHeader {
    pub entry: u64,
    pub program_header_offset: u64,
    pub program_header_entry_size: u16,
    pub program_header_count: u16,
}

impl ElfHeader {
    pub fn parse(data: &[u8]) -> Result<Self, ElfError> {
        if data.len() < ELF_HEADER_SIZE {
            return Err(ElfError::Truncated);
        }

        if data[0] != 0x7F
            || data[1] != b'E'
            || data[2] != b'L'
            || data[3] != b'F'
        {
            return Err(ElfError::InvalidMagic);
        }

        if data[4] != ELF_CLASS_64 {
            return Err(ElfError::UnsupportedClass);
        }

        if data[5] != ELF_DATA_LITTLE_ENDIAN {
            return Err(ElfError::UnsupportedEndian);
        }

        let elf_type = read_u16(data, 16)?;
        if elf_type != ELF_TYPE_EXEC {
            return Err(ElfError::UnsupportedType);
        }

        let machine = read_u16(data, 18)?;
        if machine != ELF_MACHINE_X86_64 {
            return Err(ElfError::UnsupportedMachine);
        }

        let entry = read_u64(data, 24)?;
        let program_header_offset = read_u64(data, 32)?;
        let program_header_entry_size = read_u16(data, 54)?;
        let program_header_count = read_u16(data, 56)?;

        if program_header_entry_size != 56 {
            return Err(ElfError::InvalidProgramHeaderSize);
        }

        Ok(Self {
            entry,
            program_header_offset,
            program_header_entry_size,
            program_header_count,
        })
    }
}

pub(super) fn read_u16(
    data: &[u8],
    offset: usize,
) -> Result<u16, ElfError> {
    let bytes = data
        .get(offset..offset + 2)
        .ok_or(ElfError::Truncated)?;

    Ok(u16::from_le_bytes([
        bytes[0],
        bytes[1],
    ]))
}

pub(super) fn read_u32(
    data: &[u8],
    offset: usize,
) -> Result<u32, ElfError> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or(ElfError::Truncated)?;

    Ok(u32::from_le_bytes([
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
    ]))
}

pub(super) fn read_u64(
    data: &[u8],
    offset: usize,
) -> Result<u64, ElfError> {
    let bytes = data
        .get(offset..offset + 8)
        .ok_or(ElfError::Truncated)?;

    Ok(u64::from_le_bytes([
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
    ]))
}