use mentacore_boot_protocol::BootInfo;

use crate::memory;
use crate::memory::physical::PhysicalFrameAllocator;

use super::framework::TestRunner;

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

    runner.run(
        b"physical::reservation",
        || test_reservation(allocator),
    );

    runner.run(
        b"physical::invalid_reservation",
        || test_invalid_reservation(allocator),
    );

    runner.run(
        b"physical::allocation_below_limit",
        || test_allocation_below_limit(allocator),
    );

    runner.run(
        b"physical::conventional_region_below_4g",
        test_conventional_region_below_4g,
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

fn test_reservation(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let frame =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    let address =
        frame.start_address;

    if allocator
        .free_frame(frame)
        .is_err()
    {
        return false;
    }

    if allocator
        .reserve_range(
            address,
            memory::paging::PAGE_SIZE,
        )
        .is_err()
    {
        return false;
    }

    let reserved =
        memory::physical::Frame {
            start_address: address,
        };

    if !allocator
        .is_frame_used(reserved)
        .unwrap_or(false)
    {
        return false;
    }

    let next =
        match allocator.allocate_frame() {
            Some(frame) => frame,
            None => return false,
        };

    next.start_address != address
}

fn test_invalid_reservation(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    if allocator
        .reserve_range(
            0x1234,
            memory::paging::PAGE_SIZE,
        )
        .is_ok()
    {
        return false;
    }

    if allocator
        .reserve_range(
            0,
            0,
        )
        .is_ok()
    {
        return false;
    }

    let out_of_range =
        match allocator
            .frame_count()
            .checked_mul(
                memory::paging::PAGE_SIZE,
            )
        {
            Some(address) => address,
            None => return false,
        };

    allocator
        .reserve_range(
            out_of_range,
            memory::paging::PAGE_SIZE,
        )
        .is_err()
}

fn test_allocation_below_limit(
    allocator: &mut PhysicalFrameAllocator,
) -> bool {
    let limit = 0x1_0000_0000u64;

    let frame = match allocator.allocate_frame_below(limit) {
        Some(frame) => frame,
        None => return false,
    };

    if frame.start_address >= limit {
        return false;
    }

    allocator.free_frame(frame).is_ok()
}

fn test_conventional_region_below_4g() -> bool {
    let descriptors = [
        memory::memory_map::MemoryDescriptor {
            ty: 7,
            pad: 0,
            physical_start: 0x1_0000_0000,
            virtual_start: 0,
            number_of_pages: 0x1000,
            attribute: 0,
        },
    ];

    let boot_info = BootInfo {
        version: 2,
        framebuffer_addr: 0,
        framebuffer_size: 0,
        framebuffer_width: 0,
        framebuffer_height: 0,
        framebuffer_stride: 0,
        framebuffer_format: 0,
        memory_map_addr: descriptors.as_ptr() as u64,
        memory_map_size:
            core::mem::size_of_val(&descriptors) as u64,
        memory_map_descriptor_size:
            core::mem::size_of::<memory::memory_map::MemoryDescriptor>()
                as u32,
        memory_map_descriptor_version: 1,
        kernel_image_addr: 0,
        kernel_image_size: 0,
        userspace_image_addr: 0,
        userspace_image_size: 0,
    };

    let map = unsafe {
        match memory::memory_map::MemoryMap::from_boot_info(
            &boot_info,
        ) {
            Some(map) => map,
            None => return false,
        }
    };

    map.find_conventional_region(
        memory::paging::PAGE_SIZE,
    )
    .is_none()
}