use crate::memory::load_elf;
use crate::memory::paging::PAGE_SIZE;

use crate::memory::physical::{
    Frame,
    PhysicalFrameAllocator,
};

use super::super::framework::TestRunner;

const ENTRY: u64 =
    0x0000_1000_0000_0000;

fn write_u16(
    data: &mut [u8],
    offset: usize,
    value: u16,
) {
    data[offset..offset + 2]
        .copy_from_slice(
            &value.to_le_bytes()
        );
}

fn write_u32(
    data: &mut [u8],
    offset: usize,
    value: u32,
) {
    data[offset..offset + 4]
        .copy_from_slice(
            &value.to_le_bytes()
        );
}

fn write_u64(
    data: &mut [u8],
    offset: usize,
    value: u64,
) {
    data[offset..offset + 8]
        .copy_from_slice(
            &value.to_le_bytes()
        );
}

fn fixture() -> [u8; 0x1006] {
    let mut data = [0u8; 0x1006];

    data[0..4]
        .copy_from_slice(
            b"\x7fELF"
        );

    data[4] = 2;
    data[5] = 1;

    write_u16(
        &mut data,
        16,
        2,
    );

    write_u16(
        &mut data,
        18,
        0x3e,
    );

    write_u32(
        &mut data,
        20,
        1,
    );

    write_u64(
        &mut data,
        24,
        ENTRY,
    );

    write_u64(
        &mut data,
        32,
        0x40,
    );

    write_u16(
        &mut data,
        54,
        56,
    );

    write_u16(
        &mut data,
        56,
        1,
    );

    let ph = 0x40;

    write_u32(
        &mut data,
        ph,
        1,
    );

    write_u32(
        &mut data,
        ph + 4,
        0x5,
    );

    write_u64(
        &mut data,
        ph + 8,
        0x1000,
    );

    write_u64(
        &mut data,
        ph + 16,
        ENTRY,
    );

    write_u64(
        &mut data,
        ph + 24,
        0,
    );

    write_u64(
        &mut data,
        ph + 32,
        6,
    );

    write_u64(
        &mut data,
        ph + 40,
        0x1000,
    );

    write_u64(
        &mut data,
        ph + 48,
        0x1000,
    );

    data[0x1000..0x1006]
        .copy_from_slice(
            b"FLUST!"
        );

    data
}

pub fn run(
    runner: &mut TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"elf::loader_entry",
        || test_loader_entry(allocator),
    );

    runner.run(
        b"elf::loader_segment",
        || test_loader_segment(allocator),
    );
}

fn test_loader_entry(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let data = fixture();

    let loaded =
        match unsafe {
            load_elf(
                &data,
                allocator,
            )
        } {
            Ok(value) => value,
            Err(_) => return false,
        };

    loaded.entry() == ENTRY
        && loaded.segment_count() == 1
}

fn test_loader_segment(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let data = fixture();

    let loaded =
        match unsafe {
            load_elf(
                &data,
                allocator,
            )
        } {
            Ok(value) => value,
            Err(_) => return false,
        };

    let segment =
        match loaded.segment(0) {
            Some(segment) => segment,
            None => return false,
        };

    if segment.virtual_address != ENTRY {
        return false;
    }

    if segment.page_count != 1 {
        return false;
    }

    if segment.file_size != 6 {
        return false;
    }

    if segment.memory_size != 0x1000 {
        return false;
    }

    let frame =
        Frame {
            start_address:
                segment.physical_address,
        };

    if !allocator
        .is_frame_used(frame)
        .unwrap_or(false)
    {
        return false;
    }

    let ptr =
        segment.physical_address
            as *const u8;

    for (index, expected)
        in b"FLUST!".iter().enumerate()
    {
        let actual =
            unsafe {
                ptr.add(index).read()
            };

        if actual != *expected {
            return false;
        }
    }

    for index in 6..PAGE_SIZE as usize {
        let actual =
            unsafe {
                ptr.add(index).read()
            };

        if actual != 0 {
            return false;
        }
    }

    true
}