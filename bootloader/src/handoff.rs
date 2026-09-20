use mentacore_boot_protocol::BootInfo;

use uefi::boot;
use uefi::mem::memory_map::MemoryMap;
use uefi::println;

pub fn enter_kernel(
    boot_info: &mut BootInfo,
    entry: u64,
    stack_top: u64,
) -> ! {
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
            boot_info as *const BootInfo as u64,
        );
    }
}

unsafe extern "C" {
    fn jump_to_kernel(
        entry: u64,
        stack_top: u64,
        boot_info: u64,
    ) -> !;
}