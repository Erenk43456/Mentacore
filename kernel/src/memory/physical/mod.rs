mod allocator;
mod bitmap;
mod frame;

use crate::sync::Spinlock;

pub use allocator::PhysicalFrameAllocator;
pub use bitmap::{
    bitmap_page_count,
    frame_count_for_address,
    FrameBitmap,
};

pub use frame::Frame;

pub const PAGE_SIZE: u64 = 4096;

static FRAME_ALLOCATOR: Spinlock<Option<PhysicalFrameAllocator>> =
    Spinlock::new(None);

pub fn set_frame_allocator(allocator: PhysicalFrameAllocator) {
    let mut guard = FRAME_ALLOCATOR.lock_irqsave();
    *guard = Some(allocator);
}

pub fn frame_allocator() -> &'static Spinlock<Option<PhysicalFrameAllocator>> {
    &FRAME_ALLOCATOR
}