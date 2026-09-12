use crate::memory::paging::is_user_address;

pub fn validate_user_address(address: u64) -> bool {
    is_user_address(address)
}
