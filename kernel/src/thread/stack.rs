pub const KERNEL_STACK_SIZE: u64 = 16 * 1024;
pub const KERNEL_STACK_ALIGNMENT: u64 = 16;

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