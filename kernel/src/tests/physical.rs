use crate::memory;

use super::framework::TestRunner;

use crate::memory::physical::PhysicalFrameAllocator;

pub fn run(
    runner: &mut TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"physical::frame_free",
        || test_frame_free(allocator),
    );

    runner.run(
        b"physical::frame_reuse",
        || test_frame_reuse(allocator),
    );

    runner.run(
        b"physical::counters",
        || test_counters(allocator),
    );

    runner.run(
        b"physical::double_free",
        || test_double_free(allocator),
    );

    runner.run(
        b"physical::invalid_frames",
        || test_invalid_frames(allocator),
    );

    runner.run(
        b"physical::contiguous_allocation",
        || test_contiguous_allocation(allocator),
    );
}

fn test_frame_free(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => return false,
    };

    if !allocator
        .is_frame_used(frame)
        .unwrap_or(false)
    {
        return false;
    }

    if allocator.free_frame(frame).is_err() {
        return false;
    }

    !allocator
        .is_frame_used(frame)
        .unwrap_or(false)
}

fn test_frame_reuse(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => return false,
    };

    let address = frame.start_address;

    if allocator.free_frame(frame).is_err() {
        return false;
    }

    let reused = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => return false,
    };

    reused.start_address == address
}

fn test_counters(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let live_before =
        allocator.allocated_count();

    let total_before =
        allocator.total_allocations();

    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => return false,
    };

    if allocator.allocated_count()
        != live_before + 1
    {
        return false;
    }

    if allocator.total_allocations()
        != total_before + 1
    {
        return false;
    }

    if allocator.free_frame(frame).is_err() {
        return false;
    }

    if allocator.allocated_count()
        != live_before
    {
        return false;
    }

    allocator.total_allocations()
        == total_before + 1
}

fn test_double_free(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let frame = match allocator.allocate_frame() {
        Some(frame) => frame,
        None => return false,
    };

    if allocator.free_frame(frame).is_err() {
        return false;
    }

    allocator.free_frame(frame).is_err()
}

fn test_invalid_frames(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let unaligned =
        memory::physical::Frame {
            start_address: 0x1234,
        };

    if allocator
        .free_frame(unaligned)
        .is_ok()
    {
        return false;
    }

    let out_of_range_address =
        match allocator
            .frame_count()
            .checked_mul(memory::paging::PAGE_SIZE)
        {
            Some(address) => address,
            None => return false,
        };

    let out_of_range =
        memory::physical::Frame {
            start_address: out_of_range_address,
        };

    allocator
        .free_frame(out_of_range)
        .is_err()
}

fn test_contiguous_allocation(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let before =
        allocator.allocated_count();

    let first =
        match allocator.allocate_contiguous_frames(4) {
            Some(frame) => frame,
            None => return false,
        };

    if allocator.allocated_count()
        != before + 4
    {
        return false;
    }

    let page_size =
        crate::memory::paging::PAGE_SIZE;

    for index in 0..4u64 {
        let address =
            first.start_address
                + index * page_size;

        let frame =
            crate::memory::physical::Frame {
                start_address: address,
            };

        if !allocator
            .is_frame_used(frame)
            .unwrap_or(false)
        {
            return false;
        }
    }

    true
}