use super::memory_map::MemoryMap;

const PAGE_SIZE: u64 = 4096;
const EFI_CONVENTIONAL_MEMORY: u32 = 7;

#[derive(Clone, Copy)]
pub struct Frame {
    pub start_address: u64,
}

impl Frame {
    pub fn new(start_address: u64) -> Option<Self> {
        if start_address & (PAGE_SIZE - 1) != 0 {
            return None;
        }

        Some(Self {
            start_address,
        })
    }
}

pub struct PhysicalFrameAllocator<'a> {
    memory_map: MemoryMap<'a>,
    current_descriptor: usize,
    next_frame: u64,
    remaining_frames: u64,
    allocated_frames: u64,
}

impl<'a> PhysicalFrameAllocator<'a> {
    pub fn new(memory_map: MemoryMap<'a>) -> Self {
        Self {
            memory_map,
            current_descriptor: 0,
            next_frame: 0,
            remaining_frames: 0,
            allocated_frames: 0,
        }
    }

    pub fn allocate_frame(&mut self) -> Option<Frame> {
        loop {
            if self.remaining_frames > 0 {
                let frame = Frame::new(self.next_frame)?;

                self.next_frame += PAGE_SIZE;
                self.remaining_frames -= 1;

                self.allocated_frames += 1;

                return Some(frame);
            }

            if self.current_descriptor >= self.memory_map.descriptor_count() {
                return None;
            }

            let descriptor = unsafe {
                self.memory_map
                    .descriptor(self.current_descriptor)?
            };

            self.current_descriptor += 1;

            if descriptor.ty != EFI_CONVENTIONAL_MEMORY {
                continue;
            }

            self.next_frame = descriptor.physical_start;
            self.remaining_frames = descriptor.number_of_pages;

            if self.next_frame == 0 && self.remaining_frames > 0 {
                self.next_frame += PAGE_SIZE;
                self.remaining_frames -= 1;
            }
        }
    }

    pub fn allocated_count(&self) -> u64 {
        self.allocated_frames
    }
}