mod header;
mod parser;
mod program;
mod loader;

pub use parser::ParsedElf;

pub use program::LoadSegment;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ElfError {
    Truncated,
    InvalidMagic,
    UnsupportedClass,
    UnsupportedEndian,
    UnsupportedType,
    UnsupportedMachine,
    InvalidProgramHeaderSize,
    ProgramHeadersOutOfBounds,
    FileRangeOutOfBounds,
    InvalidSegmentSize,
    InvalidAlignment,
    NonCanonicalAddress,
    IntegerOverflow,
    TooManyLoadSegments,
    NoLoadSegments,
    EntryNotInLoadSegment,
}

pub use loader::{
    load,
    LoadedElf,
};