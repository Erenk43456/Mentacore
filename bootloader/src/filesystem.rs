#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;

use uefi::boot;
use uefi::fs::FileSystem;
use uefi::CString16;

pub const KERNEL_PATH: &str = "\\kernel.elf";
pub const USERSPACE_PATH: &str = "\\userspace.elf";

pub fn load_kernel() -> Result<Vec<u8>, ()> {
    read_file(KERNEL_PATH)
}

pub fn load_userspace() -> Result<Vec<u8>, ()> {
    read_file(USERSPACE_PATH)
}

fn read_file(path: &str) -> Result<Vec<u8>, ()> {
    let fs = boot::get_image_file_system(boot::image_handle())
        .map_err(|_| ())?;

    let mut fs = FileSystem::new(fs);

    let path = CString16::try_from(path)
        .map_err(|_| ())?;

    fs.read(path.as_ref())
        .map_err(|_| ())
}