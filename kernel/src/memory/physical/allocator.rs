use super::{
    Frame,
    FrameBitmap,
    PAGE_SIZE,
};

pub struct PhysicalFrameAllocator {
    bitmap: FrameBitmap,
    current_frame: u64,
    live_allocated_frames: u64,
    total_allocations: u64,
}

impl PhysicalFrameAllocator {
    pub fn new(bitmap: FrameBitmap) -> Self {
        Self {
            bitmap,
            current_frame: 0,
            live_allocated_frames: 0,
            total_allocations: 0,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        let frame_count = self.bitmap.frame_count();

        if frame_count == 0 {
            return None;
        }

        let start_frame =
            self.current_frame % frame_count;

        for offset in 0..frame_count {
            let frame_number =
                start_frame.checked_add(offset)?;

            if frame_number >= frame_count {
                break;
            }

            let used = unsafe {
                self.bitmap.is_used(frame_number)
            };

            if used {
                continue;
            }

            let address =
                frame_number.checked_mul(PAGE_SIZE)?;

            let frame = Frame::new(address)?;

            unsafe {
                self.bitmap.set(frame_number);
            }

            self.current_frame =
                (frame_number + 1) % frame_count;

            self.live_allocated_frames += 1;
            self.total_allocations += 1;

            return Some(frame);
        }

        for frame_number in 0..start_frame {
            let used = unsafe {
                self.bitmap.is_used(frame_number)
            };

            if used {
                continue;
            }

            let address =
                frame_number.checked_mul(PAGE_SIZE)?;

            let frame = Frame::new(address)?;

            unsafe {
                self.bitmap.set(frame_number);
            }

            self.current_frame =
                (frame_number + 1) % frame_count;

            self.live_allocated_frames += 1;
            self.total_allocations += 1;

            return Some(frame);
        }

        None
    }

    pub fn allocate_contiguous_frames(
        &mut self,
        count: usize,
    ) -> Option<Frame> {
        if count == 0 {
            return None;
        }

        let frame_count = self.bitmap.frame_count();
        let count = count as u64;

        if count > frame_count {
            return None;
        }

        for start in 0..=frame_count - count {
            let end = start + count;

            let mut available = true;

            for frame in start..end {
                if unsafe {
                    self.bitmap.is_used(frame)
                } {
                    available = false;
                    break;
                }
            }

            if !available {
                continue;
            }

            let address =
                start.checked_mul(PAGE_SIZE)?;

            let frame =
                Frame::new(address)?;

            for frame_number in start..end {
                unsafe {
                    self.bitmap.set(frame_number);
                }
            }

            self.current_frame =
                end % frame_count;

            self.live_allocated_frames += count;
            self.total_allocations += count;

            return Some(frame);
        }

        None
    }

    pub fn reserve_range(
        &mut self,
        start_address: u64,
        size: u64,
    ) -> Result<(), ()> {
        if start_address & (PAGE_SIZE - 1) != 0 {
            return Err(());
        }

        if size == 0 {
            return Err(());
        }

        let page_count =
            size
                .checked_add(PAGE_SIZE - 1)
                .ok_or(())?
                / PAGE_SIZE;

        let first_frame =
            start_address / PAGE_SIZE;

        let end_frame =
            first_frame
                .checked_add(page_count)
                .ok_or(())?;

        if end_frame > self.bitmap.frame_count() {
            return Err(());
        }

        unsafe {
            self.bitmap.mark_used_range(
                start_address,
                page_count,
            );
        }

        Ok(())
    }

    pub fn free_frame(
        &mut self,
        frame: Frame,
    ) -> Result<(), ()> {
        let address = frame.start_address;

        if address & (PAGE_SIZE - 1) != 0 {
            return Err(());
        }

        let frame_number =
            address / PAGE_SIZE;

        if frame_number >= self.bitmap.frame_count() {
            return Err(());
        }

        let used = unsafe {
            self.bitmap.is_used(frame_number)
        };

        if !used {
            return Err(());
        }

        unsafe {
            self.bitmap.clear(frame_number);
        }

        self.live_allocated_frames =
            self.live_allocated_frames
                .checked_sub(1)
                .ok_or(())?;

        if frame_number < self.current_frame {
            self.current_frame = frame_number;
        }

        Ok(())
    }

    pub fn is_frame_used(
        &self,
        frame: Frame,
    ) -> Result<bool, ()> {
        let address = frame.start_address;

        if address & (PAGE_SIZE - 1) != 0 {
            return Err(());
        }

        let frame_number =
            address / PAGE_SIZE;

        if frame_number >= self.bitmap.frame_count() {
            return Err(());
        }

        Ok(unsafe {
            self.bitmap.is_used(frame_number)
        })
    }

    pub fn allocated_count(&self) -> u64 {
        self.live_allocated_frames
    }

    pub fn total_allocations(&self) -> u64 {
        self.total_allocations
    }

    pub fn frame_count(&self) -> u64 {
        self.bitmap.frame_count()
    }
}