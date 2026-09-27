use mentacore_boot_protocol::BootInfo;

use uefi::boot;
use uefi::mem::memory_map::MemoryMap;
use uefi::println;

pub fn enter_kernel(
    boot_info_addr: u64,
    pml4_address: u64,
    entry: u64,
    stack_top: u64,
) -> ! {
    println!("Preparing kernel handoff...");

    let memory_map_storage =
        match crate::memory::allocate_memory_map() {
            Ok(storage) => storage,
            Err(_) => loop {
                core::hint::spin_loop();
            }
        };
    println!("Memory map storage: {:#018x}", memory_map_storage.address);

    println!("Exiting UEFI boot services...");

    let memory_map = unsafe {
        boot::exit_boot_services(None)
    };

    let memory_map_meta = memory_map.meta();

    let memory_map_size = memory_map_meta.map_size as usize;

    if memory_map_size > memory_map_storage.capacity {
        println!("ERROR: Memory map storage is too small.");

        loop {
            core::hint::spin_loop();
        }
    }

    unsafe {
        core::ptr::copy_nonoverlapping(
            memory_map.buffer().as_ptr(),
            memory_map_storage.address as *mut u8,
            memory_map_size,
        );
    }

    let boot_info =
        unsafe { &mut *(boot_info_addr as *mut BootInfo) };

    boot_info.memory_map_addr =
        memory_map_storage.address;

    boot_info.memory_map_size =
        memory_map_size as u64;

    boot_info.memory_map_descriptor_size =
        memory_map_meta.desc_size as u32;

    boot_info.memory_map_descriptor_version =
        memory_map_meta.desc_version;

    unsafe {
        jump_to_kernel(
            entry,
            stack_top,
            boot_info_addr,
            pml4_address,
        );
    }
}

unsafe extern "C" {
    fn jump_to_kernel(
        entry: u64,
        stack_top: u64,
        boot_info: u64,
        pml4_address: u64,
    ) -> !;
}