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
}