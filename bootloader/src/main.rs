#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

use elf::endian::AnyEndian;
use elf::ElfBytes;
use elf::abi::PT_LOAD;

use uefi::prelude::*;
use uefi::boot;
use uefi::fs::FileSystem;
use uefi::{CString16, println};

const KERNEL_PATH: &str = "\\kernel.elf";

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

    for segment in elf.segments().unwrap().iter() {
        if segment.p_type != PT_LOAD {
            continue;
        }

        println!(
            "LOAD: vaddr={:#018x} memsz={:#x}",
            segment.p_vaddr,
            segment.p_memsz
        );
    }

    println!();
    println!("Kernel ELF parsed.");
    println!("Kernel handoff not implemented yet.");
    println!();

    loop {
        core::hint::spin_loop();
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