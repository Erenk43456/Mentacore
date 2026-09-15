use crate::memory::physical::{
    Frame,
    PhysicalFrameAllocator,
    PAGE_SIZE,
};

pub const KERNEL_STACK_SIZE: u64 = 16 * 1024;
pub const KERNEL_STACK_ALIGNMENT: u64 = 16;
pub const KERNEL_STACK_PAGES: usize =
    (KERNEL_STACK_SIZE / PAGE_SIZE) as usize;

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

        let top = match base.checked_add(KERNEL_STACK_SIZE) {
            Some(top) => top,
            None => {
                for index in 0..KERNEL_STACK_PAGES {
                    let address =
                        base + index as u64 * PAGE_SIZE;

                    let frame =
                        Frame::new(address)
                            .expect("kernel stack frame must be aligned");

                    let _ =
                        allocator.free_frame(frame);
                }

                return None;
            }
        };

        match Self::new(base, top) {
            Some(stack) => Some(stack),

            None => {
                for index in 0..KERNEL_STACK_PAGES {
                    let address =
                        base + index as u64 * PAGE_SIZE;

                    let frame =
                        Frame::new(address)
                            .expect("kernel stack frame must be aligned");

                    let _ =
                        allocator.free_frame(frame);
                }

                None
            }
        }
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

impl Drop for KernelStack {
    fn drop(&mut self) {
        let mut guard =
            crate::memory::physical::frame_allocator().lock();

        let allocator =
            match guard.as_mut() {
                Some(allocator) => allocator,
                None => return,
            };

        for index in 0..KERNEL_STACK_PAGES {
            let address =
                self.base + index as u64 * PAGE_SIZE;

            let frame =
                match Frame::new(address) {
                    Some(frame) => frame,
                    None => continue,
                };

            let _ =
                allocator.free_frame(frame);
        }
    }
}