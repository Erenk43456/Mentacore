use crate::memory::paging::{
    current_pml4,
    is_user_address,
    is_user_page_mapped,
    PAGE_SIZE,
};

pub fn validate_user_address(
    address: u64,
) -> bool {
    is_user_address(address)
}

pub fn validate_user_buffer(
    address: u64,
    length: u64,
) -> bool {
    if length == 0 {
        return true;
    }

    if !is_user_address(address) {
        return false;
    }

    let end = match address.checked_add(
        length - 1,
    ) {
        Some(end) => end,
        None => return false,
    };

    if !is_user_address(end) {
        return false;
    }

    let first_page =
        address & !(PAGE_SIZE - 1);

    let last_page =
        end & !(PAGE_SIZE - 1);

    let pml4 = unsafe {
        current_pml4()
    };

    let mut page = first_page;

    loop {
        let mapped = unsafe {
            is_user_page_mapped(
                pml4,
                page,
            )
        };

        if !mapped {
            return false;
        }

        if page == last_page {
            break;
        }

        page += PAGE_SIZE;
    }

    true
}