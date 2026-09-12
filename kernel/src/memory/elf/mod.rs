mod header;
mod parser;
mod program;

pub use header::ElfHeader;

pub use parser::{
    ParsedElf,
    MAX_LOAD_SEGMENTS,
};

pub use program::{
    LoadSegment,
    SegmentFlags,
    PF_R,
    PF_W,
    PF_X,
    PT_LOAD,
};

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