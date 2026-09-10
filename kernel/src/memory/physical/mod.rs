mod allocator;
mod bitmap;
mod frame;

pub use allocator::PhysicalFrameAllocator;
pub use bitmap::{
    bitmap_page_count,
    bitmap_size_bytes,
    frame_count_for_address,
    FrameBitmap,
};
pub use frame::Frame;

pub const PAGE_SIZE: u64 = 4096;