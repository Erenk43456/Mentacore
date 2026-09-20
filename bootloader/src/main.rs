#![no_std]
#![no_main]

mod elf;
mod filesystem;
mod framebuffer;
mod handoff;
mod memory;

extern crate alloc;

use uefi::prelude::*;
use uefi::println;

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

    let kernel = match filesystem::load_kernel() {
        Ok(kernel) => kernel,
        Err(_) => {
            println!("ERROR: Failed to read kernel.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    println!("Kernel file loaded: {} bytes", kernel.len());

    let kernel_elf = match elf::KernelElf::parse(&kernel) {
        Ok(elf) => elf,
        Err(_) => loop {
            core::hint::spin_loop();
        }
    };

    let entry = kernel_elf.entry();

    println!("Kernel entry: {:#018x}", entry);
    println!();

    // ------------------------------------------------------------
    // Determine complete kernel memory range.
    // ------------------------------------------------------------

    let (kernel_start, kernel_size, kernel_pages) =
        match kernel_elf.memory_range() {
            Ok(range) => range,
            Err(_) => loop {
                core::hint::spin_loop();
            }
        };

    println!(
        "Kernel memory range: {:#018x} - {:#018x}",
        kernel_start,
        kernel_start + kernel_size
    );

    println!("Kernel pages: {}", kernel_pages);

    // ------------------------------------------------------------
    // Allocate complete kernel image.
    // ------------------------------------------------------------

    if memory::allocate_kernel(kernel_start, kernel_pages).is_err() {
        loop {
            core::hint::spin_loop();
        }
    }

    // ------------------------------------------------------------
    // Load PT_LOAD segments.
    // ------------------------------------------------------------

    if kernel_elf.load_segments().is_err() {
        loop {
            core::hint::spin_loop();
        }
    }

    println!();
    println!("All kernel segments loaded.");

    // ------------------------------------------------------------
    // Load userspace ELF.
    // ------------------------------------------------------------

    println!();
    println!("Loading userspace...");

    let userspace = match filesystem::load_userspace() {
        Ok(userspace) => userspace,
        Err(_) => {
            println!("ERROR: Failed to read userspace ELF.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    println!(
        "Userspace ELF loaded: {} bytes",
        userspace.len()
    );

    // ------------------------------------------------------------
    // Userspace image
    // ------------------------------------------------------------

    let userspace_image = match memory::allocate_userspace(&userspace) {
        Ok(image) => image,
        Err(_) => loop {
            core::hint::spin_loop();
        }
    };

    let userspace_image_addr = userspace_image.address;
    let userspace_size = userspace_image.size;

    // ------------------------------------------------------------
    // Allocate kernel stack.
    // ------------------------------------------------------------

    let kernel_stack = match memory::allocate_kernel_stack() {
        Ok(stack) => stack,
        Err(_) => loop {
            core::hint::spin_loop();
        }
    };

    let stack_base = kernel_stack.base;
    let stack_top = kernel_stack.top;

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

    let mut boot_info = match framebuffer::initialize() {
        Ok(info) => info,
        Err(_) => {
            println!("ERROR: Failed to initialize framebuffer.");

            loop {
                core::hint::spin_loop();
            }
        }
    };

    boot_info.kernel_image_addr = kernel_start;
    boot_info.kernel_image_size = kernel_size;

    boot_info.userspace_image_addr = userspace_image_addr;

    println!(
        "Userspace image address: {:#018x}",
        boot_info.userspace_image_addr
    );

    boot_info.userspace_image_size = userspace_size;

    println!(
        "Userspace image size: {} bytes",
        boot_info.userspace_image_size
    );

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
    // Kernel handoff.
    // ------------------------------------------------------------

    handoff::enter_kernel(
        &mut boot_info,
        entry,
        stack_top,
    );
}