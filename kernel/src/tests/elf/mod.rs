use crate::memory::{
    ElfError,
    ParsedElf,
    PF_R,
    PF_X,
};

use super::framework::TestRunner;

const ELF_SIZE: usize = 0x1006;

const ENTRY: u64 = 0x0000_0010_0000_0000;

const CODE: [u8; 6] = [
    0x90, // nop
    0x90, // nop
    0x90, // nop
    0x90, // nop
    0xEB, // jmp
    0xFE, // -2
];

fn write_u16(data: &mut [u8], offset: usize, value: u16) {
    let bytes = value.to_le_bytes();
    data[offset..offset + 2].copy_from_slice(&bytes);
}

fn write_u32(data: &mut [u8], offset: usize, value: u32) {
    let bytes = value.to_le_bytes();
    data[offset..offset + 4].copy_from_slice(&bytes);
}

fn write_u64(data: &mut [u8], offset: usize, value: u64) {
    let bytes = value.to_le_bytes();
    data[offset..offset + 8].copy_from_slice(&bytes);
}

fn valid_elf() -> [u8; ELF_SIZE] {
    let mut data = [0u8; ELF_SIZE];

    // ELF identification.
    data[0..4].copy_from_slice(b"\x7FELF");
    data[4] = 2; // ELF64
    data[5] = 1; // little endian
    data[6] = 1; // ELF version

    // ELF header.
    write_u16(&mut data, 16, 2); // ET_EXEC
    write_u16(&mut data, 18, 0x3E); // x86_64
    write_u32(&mut data, 20, 1); // version

    write_u64(&mut data, 24, ENTRY); // entry
    write_u64(&mut data, 32, 64); // e_phoff
    write_u64(&mut data, 40, 0); // e_shoff

    write_u32(&mut data, 48, 0); // flags

    write_u16(&mut data, 52, 64); // e_ehsize
    write_u16(&mut data, 54, 56); // e_phentsize
    write_u16(&mut data, 56, 1); // e_phnum

    // Program header at 0x40.
    let ph = 0x40;

    write_u32(&mut data, ph, 1); // PT_LOAD
    write_u32(&mut data, ph + 4, PF_R | PF_X);

    write_u64(&mut data, ph + 8, 0x1000); // offset
    write_u64(&mut data, ph + 16, ENTRY); // virtual address
    write_u64(&mut data, ph + 24, ENTRY); // physical address

    write_u64(&mut data, ph + 32, CODE.len() as u64); // filesz
    write_u64(&mut data, ph + 40, CODE.len() as u64); // memsz
    write_u64(&mut data, ph + 48, 0x1000); // alignment

    // Segment payload.
    data[0x1000..0x1000 + CODE.len()]
        .copy_from_slice(&CODE);

    data
}

fn test_valid_elf() -> bool {
    let data = valid_elf();

    let elf = match ParsedElf::parse(&data) {
        Ok(elf) => elf,
        Err(_) => return false,
    };

    elf.entry() == ENTRY
        && elf.segment_count() == 1
}

fn test_load_segment() -> bool {
    let data = valid_elf();

    let elf = match ParsedElf::parse(&data) {
        Ok(elf) => elf,
        Err(_) => return false,
    };

    let segment = match elf.segment(0) {
        Some(segment) => segment,
        None => return false,
    };

    segment.offset == 0x1000
        && segment.virtual_address == ENTRY
        && segment.physical_address == ENTRY
        && segment.file_size == 6
        && segment.memory_size == 6
        && segment.flags.readable
        && !segment.flags.writable
        && segment.flags.executable
        && segment.alignment == 0x1000
}

fn test_segment_data() -> bool {
    let data = valid_elf();

    let elf = match ParsedElf::parse(&data) {
        Ok(elf) => elf,
        Err(_) => return false,
    };

    let segment = match elf.segment(0) {
        Some(segment) => segment,
        None => return false,
    };

    match elf.segment_data(segment) {
        Ok(segment_data) => segment_data == CODE,
        Err(_) => false,
    }
}

fn test_invalid_magic() -> bool {
    let mut data = valid_elf();
    data[0] = 0;

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::InvalidMagic)
    )
}

fn test_truncated_header() -> bool {
    let data = [0u8; 32];

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::Truncated)
    )
}

fn test_program_headers_out_of_bounds() -> bool {
    let mut data = valid_elf();

    write_u64(&mut data, 32, 0x2000);

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::ProgramHeadersOutOfBounds)
    )
}

fn test_invalid_segment_size() -> bool {
    let mut data = valid_elf();

    let ph = 0x40;

    write_u64(&mut data, ph + 32, 8); // filesz
    write_u64(&mut data, ph + 40, 6); // memsz

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::InvalidSegmentSize)
    )
}

fn test_invalid_alignment() -> bool {
    let mut data = valid_elf();

    let ph = 0x40;

    write_u64(&mut data, ph + 48, 0x3000);

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::InvalidAlignment)
    )
}

fn test_no_load_segments() -> bool {
    let mut data = valid_elf();

    let ph = 0x40;

    write_u32(&mut data, ph, 2); // PT_DYNAMIC

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::NoLoadSegments)
    )
}

fn test_entry_not_in_load_segment() -> bool {
    let mut data = valid_elf();

    let ph = 0x40;

    write_u64(&mut data, 24, ENTRY + 0x1000);

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::EntryNotInLoadSegment)
    )
}

fn test_file_range_out_of_bounds() -> bool {
    let mut data = valid_elf();

    write_u64(
        &mut data,
        0x40 + 8,
        0x2000,
    );

    matches!(
        ParsedElf::parse(&data),
        Err(ElfError::FileRangeOutOfBounds)
    )
}

pub fn run(runner: &mut TestRunner) {
    runner.run(
        b"elf::valid_header",
        test_valid_elf,
    );

    runner.run(
        b"elf::load_segment",
        test_load_segment,
    );

    runner.run(
        b"elf::segment_data",
        test_segment_data,
    );

    runner.run(
        b"elf::invalid_magic",
        test_invalid_magic,
    );

    runner.run(
        b"elf::truncated_header",
        test_truncated_header,
    );

    runner.run(
        b"elf::program_headers_out_of_bounds",
        test_program_headers_out_of_bounds,
    );

    runner.run(
        b"elf::invalid_segment_size",
        test_invalid_segment_size,
    );

    runner.run(
        b"elf::invalid_alignment",
        test_invalid_alignment,
    );

    runner.run(
        b"elf::no_load_segments",
        test_no_load_segments,
    );

    runner.run(
        b"elf::entry_not_in_load_segment",
        test_entry_not_in_load_segment,
    );

    runner.run(
        b"elf::file_range_out_of_bounds",
        test_file_range_out_of_bounds,
    );
}