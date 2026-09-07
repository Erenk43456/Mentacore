#[derive(Clone, Copy)]
pub enum BootState {
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
}