use super::{
    header::ElfHeader,
    program::{
        LoadSegment,
        PT_LOAD,
        PROGRAM_HEADER_SIZE,
    },
    ElfError,
};

pub const MAX_LOAD_SEGMENTS: usize = 16;

pub struct ParsedElf<'a> {
    data: &'a [u8],
    header: ElfHeader,
    segments: [LoadSegment; MAX_LOAD_SEGMENTS],
    segment_count: usize,
}

impl<'a> ParsedElf<'a> {
    pub fn parse(
        data: &'a [u8],
    ) -> Result<Self, ElfError> {
        let header = ElfHeader::parse(data)?;

        if !is_canonical_address(header.entry) {
            return Err(ElfError::NonCanonicalAddress);
        }

        let program_header_offset =
            usize::try_from(header.program_header_offset)
                .map_err(|_| ElfError::IntegerOverflow)?;

        let entry_size =
            usize::from(header.program_header_entry_size);

        let count =
            usize::from(header.program_header_count);

        let table_size = entry_size
            .checked_mul(count)
            .ok_or(ElfError::IntegerOverflow)?;

        let table_end = program_header_offset
            .checked_add(table_size)
            .ok_or(ElfError::IntegerOverflow)?;

        if table_end > data.len() {
            return Err(ElfError::ProgramHeadersOutOfBounds);
        }

        let mut segments =
            [LoadSegment::EMPTY; MAX_LOAD_SEGMENTS];

        let mut segment_count = 0usize;

        for index in 0..count {
            let offset = program_header_offset
                .checked_add(
                    index.checked_mul(entry_size)
                        .ok_or(ElfError::IntegerOverflow)?,
                )
                .ok_or(ElfError::IntegerOverflow)?;

            let (segment_type, segment) =
                LoadSegment::parse(data, offset)?;

            if segment_type != PT_LOAD {
                continue;
            }

            if segment_count >= MAX_LOAD_SEGMENTS {
                return Err(ElfError::TooManyLoadSegments);
            }

            validate_file_range(
                data,
                &segment,
            )?;

            validate_memory_range(
                &segment,
            )?;

            segments[segment_count] = segment;
            segment_count += 1;
        }

        if segment_count == 0 {
            return Err(ElfError::NoLoadSegments);
        }

        if !entry_belongs_to_load_segment(
            header.entry,
            &segments,
            segment_count,
        ) {
            return Err(ElfError::EntryNotInLoadSegment);
        }

        Ok(Self {
            data,
            header,
            segments,
            segment_count,
        })
    }

    pub fn entry(&self) -> u64 {
        self.header.entry
    }

    pub fn segment_count(&self) -> usize {
        self.segment_count
    }

    pub fn segment(
        &self,
        index: usize,
    ) -> Option<&LoadSegment> {
        if index >= self.segment_count {
            None
        } else {
            Some(&self.segments[index])
        }
    }

    pub fn segment_data(
        &self,
        segment: &LoadSegment,
    ) -> Result<&'a [u8], ElfError> {
        let start =
            usize::try_from(segment.offset)
                .map_err(|_| ElfError::IntegerOverflow)?;

        let size =
            usize::try_from(segment.file_size)
                .map_err(|_| ElfError::IntegerOverflow)?;

        let end = start
            .checked_add(size)
            .ok_or(ElfError::IntegerOverflow)?;

        self.data
            .get(start..end)
            .ok_or(ElfError::FileRangeOutOfBounds)
    }
}

fn validate_file_range(
    data: &[u8],
    segment: &LoadSegment,
) -> Result<(), ElfError> {
    let start =
        usize::try_from(segment.offset)
            .map_err(|_| ElfError::IntegerOverflow)?;

    let size =
        usize::try_from(segment.file_size)
            .map_err(|_| ElfError::IntegerOverflow)?;

    let end = start
        .checked_add(size)
        .ok_or(ElfError::IntegerOverflow)?;

    if end > data.len() {
        return Err(ElfError::FileRangeOutOfBounds);
    }

    Ok(())
}

fn validate_memory_range(
    segment: &LoadSegment,
) -> Result<(), ElfError> {
    let end = segment
        .virtual_address
        .checked_add(segment.memory_size)
        .ok_or(ElfError::IntegerOverflow)?;

    if segment.memory_size != 0
        && !is_canonical_address(end - 1)
    {
        return Err(ElfError::NonCanonicalAddress);
    }

    Ok(())
}

fn entry_belongs_to_load_segment(
    entry: u64,
    segments: &[LoadSegment; MAX_LOAD_SEGMENTS],
    count: usize,
) -> bool {
    for segment in segments.iter().take(count) {
        let Some(end) = segment
            .virtual_address
            .checked_add(segment.memory_size)
        else {
            continue;
        };

        if entry >= segment.virtual_address
            && entry < end
        {
            return true;
        }
    }

    false
}

fn is_canonical_address(address: u64) -> bool {
    let upper = address >> 48;

    upper == 0
        || upper == 0xFFFF
}