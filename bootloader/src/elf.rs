use core::ptr;

use elf::abi::PT_LOAD;
use elf::endian::AnyEndian;
use elf::ElfBytes;

use uefi::println;

const PAGE_SIZE: u64 = 4096;

pub struct KernelElf<'a> {
    elf: ElfBytes<'a, AnyEndian>,
}

impl<'a> KernelElf<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, ()> {
        let elf = match ElfBytes::<AnyEndian>::minimal_parse(data) {
            Ok(elf) => elf,
            Err(_) => {
                println!("ERROR: Invalid ELF.");

                return Err(());
            }
        };

        Ok(Self { elf })
    }

    pub fn entry(&self) -> u64 {
        self.elf.ehdr.e_entry
    }

    pub fn memory_range(&self) -> Result<(u64, u64, usize), ()> {
        let segments = match self.elf.segments() {
            Some(segments) => segments,
            None => {
                println!("ERROR: ELF has no program headers.");

                return Err(());
            }
        };

        let mut kernel_start = u64::MAX;
        let mut kernel_end = 0u64;
        let mut load_segment_count = 0usize;

        for segment in segments.iter() {
            if segment.p_type != PT_LOAD {
                continue;
            }

            load_segment_count += 1;

            if segment.p_memsz < segment.p_filesz {
                println!("ERROR: Invalid PT_LOAD sizes.");

                return Err(());
            }

            let segment_end =
                match segment.p_vaddr.checked_add(segment.p_memsz) {
                    Some(end) => end,
                    None => {
                        println!("ERROR: Segment address overflow.");

                        return Err(());
                    }
                };

            let page_start =
                segment.p_vaddr & !(PAGE_SIZE - 1);

            let page_end = match align_up(segment_end, PAGE_SIZE) {
                Some(end) => end,
                None => {
                    println!("ERROR: Segment alignment overflow.");

                    return Err(());
                }
            };

            if page_start < kernel_start {
                kernel_start = page_start;
            }

            if page_end > kernel_end {
                kernel_end = page_end;
            }
        }

        if load_segment_count == 0 {
            println!("ERROR: ELF has no PT_LOAD segments.");

            return Err(());
        }

        if kernel_end <= kernel_start {
            println!("ERROR: Invalid kernel memory range.");

            return Err(());
        }

        let kernel_size = kernel_end - kernel_start;

        let kernel_pages =
            match usize::try_from(kernel_size / PAGE_SIZE) {
                Ok(pages) => pages,
                Err(_) => {
                    println!("ERROR: Kernel page count overflow.");

                    return Err(());
                }
            };

        Ok((kernel_start, kernel_size, kernel_pages))
    }

    pub fn load_segments(&self) -> Result<(), ()> {
        let segments = self.elf.segments().ok_or(())?;

        for segment in segments.iter() {
            if segment.p_type != PT_LOAD {
                continue;
            }

            let data = match self.elf.segment_data(&segment) {
                Ok(data) => data,
                Err(_) => {
                    println!("ERROR: Failed to read ELF segment.");

                    return Err(());
                }
            };

            let destination = segment.p_vaddr as *mut u8;

            unsafe {
                ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    destination,
                    data.len(),
                );

                if segment.p_memsz > segment.p_filesz {
                    ptr::write_bytes(
                        destination.add(segment.p_filesz as usize),
                        0,
                        (segment.p_memsz - segment.p_filesz) as usize,
                    );
                }
            }
        }

        Ok(())
    }
}

fn align_up(value: u64, alignment: u64) -> Option<u64> {
    let mask = alignment - 1;

    value
        .checked_add(mask)
        .map(|value| value & !mask)
}