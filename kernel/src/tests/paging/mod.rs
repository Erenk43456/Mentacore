mod address_space;
mod layout;
mod mapping;

use crate::memory::physical::PhysicalFrameAllocator;

use super::framework::TestRunner;

pub fn run(
    runner: &mut TestRunner,
    allocator: &mut PhysicalFrameAllocator,
) {
    runner.run(
        b"paging::duplicate_mapping",
        || mapping::test_duplicate_mapping(allocator),
    );

    runner.run(
        b"paging::mapping",
        || mapping::test_mapping(allocator),
    );

    runner.run(
        b"paging::unmap",
        || mapping::test_unmap(allocator),
    );

    runner.run(
        b"paging::unmap_rejection",
        || mapping::test_unmap_rejection(allocator),
    );

    runner.run(
        b"paging::user_mapping_permissions",
        || mapping::test_user_mapping_permissions(allocator),
    );

    runner.run(
        b"paging::virtual_address_layout",
        layout::test_virtual_address_layout,
    );

    runner.run(
        b"paging::address_space",
        address_space::test_address_space_abstraction,
    );

    runner.run(
        b"paging::address_space_mapping",
        || address_space::test_address_space_mapping(allocator),
    );

    runner.run(
        b"paging::address_space_unmapping",
        || address_space::test_address_space_unmapping(allocator),
    );
}