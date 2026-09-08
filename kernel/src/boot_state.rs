use mentacore_boot_protocol::BootInfo;

#[derive(Clone, Copy)]
pub enum BootState {
    #[allow(dead_code)]
    Initializing,

    Ready,
    Warning,
    Error,
}

impl BootState {
    pub fn status(self) -> crate::display::Status {
        match self {
            Self::Initializing => crate::display::Status::Initializing,
            Self::Ready => crate::display::Status::Ready,
            Self::Warning => crate::display::Status::Warning,
            Self::Error => crate::display::Status::Error,
        }
    }

    pub fn from_boot_info(boot_info: &BootInfo) -> Self {
        if boot_info.framebuffer_addr == 0
            || boot_info.framebuffer_size == 0
            || boot_info.framebuffer_width == 0
            || boot_info.framebuffer_height == 0
            || boot_info.framebuffer_stride == 0
        {
            return Self::Error;
        }

        if boot_info.framebuffer_format > 3 {
            return Self::Warning;
        }

        Self::Ready
    }
}