pub(super) fn test_virtual_address_layout() -> bool {
    use crate::memory::paging::{
        KERNEL_SPACE_END,
        KERNEL_SPACE_START,
        USER_SPACE_END,
        USER_SPACE_START,
    };

    if USER_SPACE_START > USER_SPACE_END {
        return false;
    }

    if KERNEL_SPACE_START > KERNEL_SPACE_END {
        return false;
    }

    if USER_SPACE_END >= KERNEL_SPACE_START {
        return false;
    }

    if !crate::memory::paging::is_user_address(
        USER_SPACE_START,
    ) {
        return false;
    }

    if !crate::memory::paging::is_user_address(
        USER_SPACE_END,
    ) {
        return false;
    }

    if !crate::memory::paging::is_kernel_address(
        KERNEL_SPACE_START,
    ) {
        return false;
    }

    if !crate::memory::paging::is_kernel_address(
        KERNEL_SPACE_END,
    ) {
        return false;
    }

    let user_kernel_boundary =
        0x0000_8000_0000_0000;

    if crate::memory::paging::is_user_address(
        user_kernel_boundary,
    ) {
        return false;
    }

    if crate::memory::paging::is_kernel_address(
        user_kernel_boundary,
    ) {
        return false;
    }

    true
}