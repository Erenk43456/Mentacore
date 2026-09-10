const PAGE_SIZE: u64 = 4096;

#[derive(Clone, Copy)]
pub struct Frame {
    pub start_address: u64,
}

impl Frame {
    pub fn new(start_address: u64) -> Option<Self> {
        if start_address & (PAGE_SIZE - 1) != 0 {
            return None;
        }

        Some(Self { start_address })
    }
}