use mentacore_boot_protocol::BootInfo;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryDescriptor {
    pub ty: u32,
    pub pad: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attribute: u64,
}

pub struct MemoryMap<'a> {
    boot_info: &'a BootInfo,
}

impl<'a> MemoryMap<'a> {
    pub unsafe fn from_boot_info(boot_info: &'a BootInfo) -> Option<Self> {
        if boot_info.memory_map_addr == 0
            || boot_info.memory_map_size == 0
            || boot_info.memory_map_descriptor_size == 0
        {
            return None;
        }

        if boot_info.memory_map_size < boot_info.memory_map_descriptor_size as u64 {
            return None;
        }

        Some(Self { boot_info })
    }

    pub fn descriptor_count(&self) -> usize {
        self.boot_info.memory_map_size as usize
            / self.boot_info.memory_map_descriptor_size as usize
    }

    pub unsafe fn descriptor(&self, index: usize) -> Option<MemoryDescriptor> {
        if index >= self.descriptor_count() {
            return None;
        }

        let base = self.boot_info.memory_map_addr as *const u8;

        let offset =
            index * self.boot_info.memory_map_descriptor_size as usize;

        let ptr = unsafe {
            base.add(offset) as *const MemoryDescriptor
        };

        Some(unsafe {
            ptr.read_unaligned()
        })
    }

    pub fn highest_conventional_address(&self) -> Option<u64> {
        let mut highest: Option<u64> = None;

        for index in 0..self.descriptor_count() {
            let descriptor = unsafe {
                self.descriptor(index)?
            };

            if descriptor.ty != 7 {
                continue;
            }

            let end = descriptor
                .physical_start
                .checked_add(
                    descriptor.number_of_pages.checked_mul(4096)?
                )?;

            highest = Some(
                highest.map_or(end, |current| current.max(end))
            );
        }

        highest
    }

    pub fn find_conventional_region(
        &self,
        required_size: u64,
    ) -> Option<u64> {
        if required_size == 0 {
            return None;
        }

        let required_size =
            (required_size + 4095) & !4095;

        // Physical address 0 is intentionally avoided.
        const MIN_ADDRESS: u64 = 0x0010_0000;

        for index in 0..self.descriptor_count() {
            let descriptor = unsafe {
                self.descriptor(index)?
            };

            if descriptor.ty != 7 {
                continue;
            }

            let region_start =
                descriptor.physical_start.max(MIN_ADDRESS);

            let region_end =
                descriptor
                    .physical_start
                    .checked_add(
                        descriptor.number_of_pages.checked_mul(4096)?
                    )?;

            if region_end <= region_start {
                continue;
            }

            let available_size =
                region_end.checked_sub(region_start)?;

            if available_size < required_size {
                continue;
            }

            let bitmap_end =
                region_start.checked_add(required_size)?;

            if bitmap_end <= region_end {
                return Some(region_start);
            }
        }

        None
    }
}