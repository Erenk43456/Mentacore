#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use core::ptr;

use elf::abi::PT_LOAD;
use elf::endian::AnyEndian;
use elf::ElfBytes;

use mentacore_boot_protocol::{BootInfo, BOOT_PROTOCOL_VERSION};

use uefi::boot::{self, AllocateType, MemoryType};
use uefi::mem::memory_map::MemoryMap;
use uefi::fs::FileSystem;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::{CString16, println};

const KERNEL_PATH: &str = "\\kernel.elf";
const PAGE_SIZE: u64 = 4096;
const KERNEL_STACK_PAGES: usize = 4;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    println!();
    println!("================================");
    println!("       MENTACORE BOOTLOADER");
    println!("================================");
    println!();

    println!("UEFI initialized.");
    println!("Bootloader entry reached.");
    println!();

    // ------------------------------------------------------------
    // Load kernel ELF.
    // ------------------------------------------------------------

    println!("Loading kernel...");

    let kernel = match load_kernel_file() {
        Ok(kernel) => kernel,
        Err(_) => {
            println!("ERROR: Failed to read kernel.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    println!("Kernel file loaded: {} bytes", kernel.len());

    let elf = match ElfBytes::<AnyEndian>::minimal_parse(&kernel) {
        Ok(elf) => elf,
        Err(_) => {
            println!("ERROR: Invalid ELF.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    let entry = elf.ehdr.e_entry;

    println!("Kernel entry: {:#018x}", entry);
    println!();

    let segments = match elf.segments() {
        Some(segments) => segments,
        None => {
            println!("ERROR: ELF has no program headers.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    // ------------------------------------------------------------
    // Determine complete kernel memory range.
    // ------------------------------------------------------------

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

            loop {
                core::hint::spin_loop();
            }
        }

        let segment_end = match segment.p_vaddr.checked_add(segment.p_memsz) {
            Some(end) => end,
            None => {
                println!("ERROR: Segment address overflow.");

                loop {
                    core::hint::spin_loop();
                }
            }
        };

        let page_start = segment.p_vaddr & !(PAGE_SIZE - 1);

        let page_end = match align_up(segment_end, PAGE_SIZE) {
            Some(end) => end,
            None => {
                println!("ERROR: Segment alignment overflow.");

                loop {
                    core::hint::spin_loop();
                }
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

        loop {
            core::hint::spin_loop();
        }
    }

    if kernel_end <= kernel_start {
        println!("ERROR: Invalid kernel memory range.");

        loop {
            core::hint::spin_loop();
        }
    }

    let kernel_size = kernel_end - kernel_start;

    let kernel_pages = match usize::try_from(kernel_size / PAGE_SIZE) {
        Ok(pages) => pages,
        Err(_) => {
            println!("ERROR: Kernel page count overflow.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    println!(
        "Kernel memory range: {:#018x} - {:#018x}",
        kernel_start,
        kernel_end
    );

    println!("Kernel pages: {}", kernel_pages);

    // ------------------------------------------------------------
    // Allocate complete kernel image.
    // ------------------------------------------------------------

    println!("Allocating kernel memory...");

    let allocation = match boot::allocate_pages(
        AllocateType::Address(kernel_start.into()),
        MemoryType::LOADER_CODE,
        kernel_pages,
    ) {
        Ok(ptr) => ptr,
        Err(_) => {
            println!("ERROR: Failed to allocate kernel pages.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    let allocated_address = allocation.as_ptr() as u64;

    if allocated_address != kernel_start {
        println!("ERROR: Kernel allocated at wrong address.");

        loop {
            core::hint::spin_loop();
        }
    }

    println!("Kernel memory allocated.");

    // ------------------------------------------------------------
    // Load PT_LOAD segments.
    // ------------------------------------------------------------

    for segment in segments.iter() {
        if segment.p_type != PT_LOAD {
            continue;
        }

        let segment_end = match segment.p_vaddr.checked_add(segment.p_memsz) {
            Some(end) => end,
            None => {
                println!("ERROR: Segment address overflow.");

                loop {
                    core::hint::spin_loop();
                }
            }
        };

        println!(
            "Loading segment: {:#018x} - {:#018x}",
            segment.p_vaddr,
            segment_end
        );

        let data = match elf.segment_data(&segment) {
            Ok(data) => data,
            Err(_) => {
                println!("ERROR: Failed to read ELF segment.");

                loop {
                    core::hint::spin_loop();
                }
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

        println!("  Segment loaded.");
    }

    println!();
    println!("All kernel segments loaded.");
    println!("Kernel entry: {:#018x}", entry);

    // ------------------------------------------------------------
    // Allocate kernel stack.
    // ------------------------------------------------------------

    println!("Allocating kernel stack...");

    let stack_allocation = match boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        KERNEL_STACK_PAGES,
    ) {
        Ok(ptr) => ptr,
        Err(_) => {
            println!("ERROR: Failed to allocate kernel stack.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    let stack_base = stack_allocation.as_ptr() as u64;

    let stack_size = match (KERNEL_STACK_PAGES as u64).checked_mul(PAGE_SIZE) {
        Some(size) => size,
        None => {
            println!("ERROR: Kernel stack size overflow.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    let stack_top = match stack_base.checked_add(stack_size) {
        Some(top) => top,
        None => {
            println!("ERROR: Kernel stack address overflow.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    println!(
        "Kernel stack: {:#018x} - {:#018x}",
        stack_base,
        stack_top
    );

    // ------------------------------------------------------------
    // Initialize framebuffer.
    // ------------------------------------------------------------

    println!();
    println!("Initializing framebuffer...");

    let mut boot_info = match get_framebuffer_info() {
        Ok(info) => info,
        Err(_) => {
            println!("ERROR: Failed to initialize framebuffer.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    println!(
        "Framebuffer: {:#018x}",
        boot_info.framebuffer_addr
    );

    println!(
        "Framebuffer size: {} bytes",
        boot_info.framebuffer_size
    );

    println!(
        "Resolution: {}x{}",
        boot_info.framebuffer_width,
        boot_info.framebuffer_height
    );

    println!(
        "Stride: {}",
        boot_info.framebuffer_stride
    );

    println!(
        "Pixel format: {}",
        boot_info.framebuffer_format
    );

    // ------------------------------------------------------------
    // Final kernel entry check.
    // ------------------------------------------------------------

    let kernel_ptr = entry as *const u8;

    unsafe {
        println!(
            "Kernel bytes at entry: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            kernel_ptr.read(),
            kernel_ptr.add(1).read(),
            kernel_ptr.add(2).read(),
            kernel_ptr.add(3).read(),
            kernel_ptr.add(4).read(),
            kernel_ptr.add(5).read(),
            kernel_ptr.add(6).read(),
            kernel_ptr.add(7).read(),
        );
    }

    // ------------------------------------------------------------
    // Kernel handoff.
    // ------------------------------------------------------------

    println!();
    println!("Preparing kernel handoff...");
    println!("Exiting UEFI boot services...");

    let memory_map = unsafe {
        boot::exit_boot_services(None)
    };

    let memory_map_meta = memory_map.meta();

    boot_info.memory_map_addr =
        memory_map.buffer().as_ptr() as u64;

    boot_info.memory_map_size =
        memory_map_meta.map_size as u64;

    boot_info.memory_map_descriptor_size =
        memory_map_meta.desc_size as u32;

    boot_info.memory_map_descriptor_version =
        memory_map_meta.desc_version;

    unsafe {
        jump_to_kernel(
            entry,
            stack_top,
            &boot_info as *const BootInfo as u64,
        );
    }
}

fn load_kernel_file() -> Result<Vec<u8>, ()> {
    let fs = boot::get_image_file_system(boot::image_handle())
        .map_err(|_| ())?;

    let mut fs = FileSystem::new(fs);

    let path = CString16::try_from(KERNEL_PATH)
        .map_err(|_| ())?;

    fs.read(path.as_ref())
        .map_err(|_| ())
}

fn get_framebuffer_info() -> Result<BootInfo, ()> {
    let handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|_| ())?;

    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(handle)
        .map_err(|_| ())?;

    const TARGET_WIDTH: usize = 1920;
    const TARGET_HEIGHT: usize = 1080;

    println!("Searching for preferred GOP mode...");
    println!(
        "Preferred resolution: {}x{}",
        TARGET_WIDTH,
        TARGET_HEIGHT
    );

    let mut selected_mode = None;

    for mode in gop.modes() {
        let info = mode.info();
        let (width, height) = info.resolution();

        if width == TARGET_WIDTH && height == TARGET_HEIGHT {
            selected_mode = Some(mode);
            break;
        }
    }

    if let Some(mode) = selected_mode {
        println!("Found preferred GOP mode.");

        gop.set_mode(&mode).map_err(|_| ())?;

        println!("GOP mode changed.");
    } else {
        println!("Preferred GOP mode not found.");
        println!("Keeping current GOP mode.");
    }

    let info = gop.current_mode_info();

    let (width, height) = info.resolution();
    let stride = info.stride();

    let mut framebuffer = gop.frame_buffer();

    let framebuffer_addr = framebuffer.as_mut_ptr() as u64;
    let framebuffer_size = framebuffer.size() as u64;

    let framebuffer_format = match info.pixel_format() {
        PixelFormat::Rgb => 0,
        PixelFormat::Bgr => 1,
        PixelFormat::Bitmask => 2,
        PixelFormat::BltOnly => 3,
    };

    Ok(BootInfo {
        version: BOOT_PROTOCOL_VERSION,

        framebuffer_addr,
        framebuffer_size,
        framebuffer_width: width as u32,
        framebuffer_height: height as u32,
        framebuffer_stride: stride as u32,
        framebuffer_format,

        memory_map_addr: 0,
        memory_map_size: 0,
        memory_map_descriptor_size: 0,
        memory_map_descriptor_version: 0,
    })
}

fn align_up(value: u64, alignment: u64) -> Option<u64> {
    let mask = alignment - 1;

    value
        .checked_add(mask)
        .map(|value| value & !mask)
}

unsafe fn jump_to_kernel(
    entry: u64,
    stack_top: u64,
    boot_info: u64,
) -> ! {
    unsafe {
        core::arch::asm!(
            "mov rsp, {stack}",
            "mov rdi, {boot_info}",
            "jmp {entry}",
            stack = in(reg) stack_top,
            boot_info = in(reg) boot_info,
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}