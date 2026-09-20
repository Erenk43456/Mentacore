use core::ptr;

use uefi::boot::{self, AllocateType, MemoryType};
use uefi::println;

const PAGE_SIZE: u64 = 4096;
const KERNEL_STACK_PAGES: usize = 4;
const MAX_USERSPACE_PHYSICAL_ADDRESS: u64 = 0xFFFF_FFFF;

pub struct UserspaceImage {
    pub address: u64,
    pub size: u64,
}

pub struct KernelStack {
    pub base: u64,
    pub top: u64,
}

pub fn allocate_kernel(
    kernel_start: u64,
    kernel_pages: usize,
) -> Result<(), ()> {
    println!("Allocating kernel memory...");

    let allocation = match boot::allocate_pages(
        AllocateType::Address(kernel_start.into()),
        MemoryType::LOADER_CODE,
        kernel_pages,
    ) {
        Ok(ptr) => ptr,
        Err(_) => {
            println!("ERROR: Failed to allocate kernel pages.");
            return Err(());
        }
    };

    let allocated_address = allocation.as_ptr() as u64;

    if allocated_address != kernel_start {
        println!("ERROR: Kernel allocated at wrong address.");
        return Err(());
    }

    println!("Kernel memory allocated.");

    Ok(())
}

pub fn allocate_userspace(userspace: &[u8]) -> Result<UserspaceImage, ()> {
    if userspace.is_empty() {
        println!("ERROR: Userspace ELF is empty.");
        return Err(());
    }

    let userspace_size = userspace.len() as u64;

    let userspace_pages_size = match align_up(userspace_size, PAGE_SIZE) {
        Some(size) => size,
        None => {
            println!("ERROR: Userspace image size overflow.");
            return Err(());
        }
    };

    let userspace_pages =
        match usize::try_from(userspace_pages_size / PAGE_SIZE) {
            Ok(pages) if pages > 0 => pages,
            _ => {
                println!("ERROR: Invalid userspace page count.");
                return Err(());
            }
        };

    println!("Userspace pages: {}", userspace_pages);
    println!("Allocating userspace image memory...");

    let userspace_allocation = match boot::allocate_pages(
        AllocateType::MaxAddress(
            MAX_USERSPACE_PHYSICAL_ADDRESS.into(),
        ),
        MemoryType::LOADER_DATA,
        userspace_pages,
    ) {
        Ok(ptr) => ptr,
        Err(_) => {
            println!("ERROR: Failed to allocate userspace image.");
            return Err(());
        }
    };

    let userspace_image_addr =
        userspace_allocation.as_ptr() as u64;

    println!(
        "Userspace image: {:#018x} - {:#018x}",
        userspace_image_addr,
        userspace_image_addr + userspace_pages_size
    );

    unsafe {
        ptr::copy_nonoverlapping(
            userspace.as_ptr(),
            userspace_image_addr as *mut u8,
            userspace.len(),
        );
    }

    println!("Userspace image copied.");

    Ok(UserspaceImage {
        address: userspace_image_addr,
        size: userspace_size,
    })
}

pub fn allocate_kernel_stack() -> Result<KernelStack, ()> {
    println!("Allocating kernel stack...");

    let stack_allocation = match boot::allocate_pages(
        AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        KERNEL_STACK_PAGES,
    ) {
        Ok(ptr) => ptr,
        Err(_) => {
            println!("ERROR: Failed to allocate kernel stack.");
            return Err(());
        }
    };

    let stack_base = stack_allocation.as_ptr() as u64;

    let stack_size =
        match (KERNEL_STACK_PAGES as u64).checked_mul(PAGE_SIZE) {
            Some(size) => size,
            None => {
                println!("ERROR: Kernel stack size overflow.");
                return Err(());
            }
        };

    let stack_top = match stack_base.checked_add(stack_size) {
        Some(top) => top,
        None => {
            println!("ERROR: Kernel stack address overflow.");
            return Err(());
        }
    };

    Ok(KernelStack {
        base: stack_base,
        top: stack_top,
    })
}

fn align_up(value: u64, alignment: u64) -> Option<u64> {
    let mask = alignment - 1;

    value
        .checked_add(mask)
        .map(|value| value & !mask)
}