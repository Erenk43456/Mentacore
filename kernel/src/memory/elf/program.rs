use super::{
    header::{read_u32, read_u64},
    ElfError,
};

pub const PT_LOAD: u32 = 1;

pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;

#[cfg(feature = "kernel-tests")]
pub const PF_R: u32 = 4;

pub const PROGRAM_HEADER_SIZE: usize = 56;

#[derive(Clone, Copy)]
pub struct SegmentFlags {
    #[cfg(feature = "kernel-tests")]
    pub readable: bool,

    pub writable: bool,
    pub executable: bool,
}

impl SegmentFlags {
    pub fn from_bits(bits: u32) -> Self {
        Self {
            #[cfg(feature = "kernel-tests")]
            readable: bits & PF_R != 0,

            writable: bits & PF_W != 0,
            executable: bits & PF_X != 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct LoadSegment {
    pub offset: u64,
    pub virtual_address: u64,

    #[cfg(feature = "kernel-tests")]
    pub physical_address: u64,

    pub file_size: u64,
    pub memory_size: u64,
    pub flags: SegmentFlags,

    #[cfg(feature = "kernel-tests")]
    pub alignment: u64,
}

impl LoadSegment {
    pub const EMPTY: Self = Self {
        offset: 0,
        virtual_address: 0,

        #[cfg(feature = "kernel-tests")]
        physical_address: 0,

        file_size: 0,
        memory_size: 0,
        flags: SegmentFlags {
            #[cfg(feature = "kernel-tests")]
            readable: false,
            writable: false,
            executable: false,
        },

        #[cfg(feature = "kernel-tests")]
        alignment: 0,
    };

    pub fn parse(
        data: &[u8],
        offset: usize,
    ) -> Result<(u32, Self), ElfError> {
        let segment_type = read_u32(data, offset)?;
        let flag_bits = read_u32(data, offset + 4)?;

        let file_offset = read_u64(data, offset + 8)?;
        let virtual_address = read_u64(data, offset + 16)?;

        #[cfg(feature = "kernel-tests")]
        let physical_address = read_u64(data, offset + 24)?;
        let file_size = read_u64(data, offset + 32)?;

        let memory_size = read_u64(data, offset + 40)?;
        let alignment = read_u64(data, offset + 48)?;

        if memory_size < file_size {
            return Err(ElfError::InvalidSegmentSize);
        }

        if alignment != 0
            && alignment != 1
            && !alignment.is_power_of_two()
        {
            return Err(ElfError::InvalidAlignment);
        }

        if alignment > 1
            && (virtual_address % alignment)
                != (file_offset % alignment)
        {
            return Err(ElfError::InvalidAlignment);
        }

        if !is_canonical_address(virtual_address) {
            return Err(ElfError::NonCanonicalAddress);
        }

        Ok((
            segment_type,
            Self {
                offset: file_offset,
                virtual_address,

                #[cfg(feature = "kernel-tests")]
                physical_address,

                file_size,
                memory_size,
                flags: SegmentFlags::from_bits(flag_bits),

                #[cfg(feature = "kernel-tests")]
                alignment,
            }
        ))
    }
}

fn is_canonical_address(address: u64) -> bool {
    let upper = address >> 48;

    upper == 0
        || upper == 0xFFFF
}