use super::PAGE_SIZE;

const BITS_PER_BYTE: u64 = 8;

pub fn frame_count_for_address(address: u64) -> Option<u64> {
    let frame_count = address
        .checked_add(PAGE_SIZE - 1)?
        / PAGE_SIZE;

    Some(frame_count)
}

pub fn bitmap_size_bytes(frame_count: u64) -> Option<u64> {
    Some(
        frame_count
            .checked_add(BITS_PER_BYTE - 1)?
            / BITS_PER_BYTE
    )
}

pub fn bitmap_page_count(frame_count: u64) -> Option<u64> {
    let bytes = bitmap_size_bytes(frame_count)?;

    bytes
        .checked_add(PAGE_SIZE - 1)
        .map(|value| value / PAGE_SIZE)
}

pub struct FrameBitmap {
    address: u64,
    frame_count: u64,
}

impl FrameBitmap {
    pub unsafe fn new(
        address: u64,
        frame_count: u64,
    ) -> Option<Self> {
        if address & (PAGE_SIZE - 1) != 0 {
            return None;
        }

        if frame_count == 0 {
            return None;
        }

        Some(Self {
            address,
            frame_count,
        })
    }

    fn byte_ptr(&self, byte_index: u64) -> *mut u8 {
        (self.address + byte_index) as *mut u8
    }

    fn bit_position(frame: u64) -> (u64, u8) {
        let byte_index = frame / BITS_PER_BYTE;
        let bit_index = (frame % BITS_PER_BYTE) as u8;

        (byte_index, bit_index)
    }

    pub unsafe fn clear_all(&mut self) {
        let byte_count =
            bitmap_size_bytes(self.frame_count)
                .unwrap_or(0);

        for index in 0..byte_count {
            unsafe {
                self.byte_ptr(index).write(0);
            }
        }
    }

    pub unsafe fn mark_free_range(
        &mut self,
        start_address: u64,
        page_count: u64,
    ) {
        let first_frame =
            start_address / PAGE_SIZE;

        let end_frame =
            match first_frame.checked_add(page_count) {
                Some(value) => value,
                None => return,
            };

        for frame_number in first_frame..end_frame {
            if frame_number >= self.frame_count {
                break;
            }

            let (byte_index, bit_index) =
                Self::bit_position(frame_number);

            unsafe {
                let ptr = self.byte_ptr(byte_index);
                let value = ptr.read();

                ptr.write(
                    value & !(1u8 << bit_index)
                );
            }
        }
    }

    pub unsafe fn mark_used_range(
        &mut self,
        start_address: u64,
        page_count: u64,
    ) {
        let first_frame =
            start_address / PAGE_SIZE;

        let end_frame =
            match first_frame.checked_add(page_count) {
                Some(value) => value,
                None => return,
            };

        for frame_number in first_frame..end_frame {
            if frame_number >= self.frame_count {
                break;
            }

            self.set(frame_number);
        }
    }

    pub unsafe fn set(&mut self, frame: u64) {
        if frame >= self.frame_count {
            return;
        }

        let (byte_index, bit_index) =
            Self::bit_position(frame);

        unsafe {
            let ptr = self.byte_ptr(byte_index);
            let value = ptr.read();

            ptr.write(
                value | (1u8 << bit_index)
            );
        }
    }

    pub unsafe fn clear(&mut self, frame: u64) {
        if frame >= self.frame_count {
            return;
        }

        let (byte_index, bit_index) =
            Self::bit_position(frame);

        unsafe {
            let ptr = self.byte_ptr(byte_index);
            let value = ptr.read();

            ptr.write(
                value & !(1u8 << bit_index)
            );
        }
    }

    pub unsafe fn is_used(&self, frame: u64) -> bool {
        if frame >= self.frame_count {
            return true;
        }

        let (byte_index, bit_index) =
            Self::bit_position(frame);

        unsafe {
            let value =
                self.byte_ptr(byte_index).read();

            (value & (1u8 << bit_index)) != 0
        }
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}