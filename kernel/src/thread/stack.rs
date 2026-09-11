use crate::memory::physical::PhysicalFrameAllocator;

pub const KERNEL_STACK_SIZE: u64 = 16 * 1024;
pub const KERNEL_STACK_ALIGNMENT: u64 = 16;
pub const KERNEL_STACK_PAGES: usize =
    (KERNEL_STACK_SIZE / crate::memory::physical::PAGE_SIZE) as usize;

#[derive(Clone, Copy)]
pub struct KernelStack {
    base: u64,
    top: u64,
}

impl KernelStack {
    pub const fn new(
        base: u64,
        top: u64,
    ) -> Option<Self> {
        if top <= base {
            return None;
        }

        if top - base != KERNEL_STACK_SIZE {
            return None;
        }

        if base % KERNEL_STACK_ALIGNMENT != 0 {
            return None;
        }

        if top % KERNEL_STACK_ALIGNMENT != 0 {
            return None;
        }

        Some(Self {
            base,
            top,
        })
    }

    pub fn allocate(
        allocator: &mut PhysicalFrameAllocator,
    ) -> Option<Self> {
        let frame =
            allocator.allocate_contiguous_frames(
                KERNEL_STACK_PAGES,
            )?;

        let base = frame.start_address;
        let top = base + KERNEL_STACK_SIZE;

        Self::new(base, top)
    }

    pub fn base(&self) -> u64 {
        self.base
    }

    pub fn top(&self) -> u64 {
        self.top
    }

    pub fn size(&self) -> u64 {
        self.top - self.base
    }
}