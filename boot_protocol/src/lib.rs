#![no_std]

pub const BOOT_PROTOCOL_VERSION: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootInfo {
    pub version: u32,

    pub framebuffer_addr: u64,
    pub framebuffer_size: u64,

    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_stride: u32,

    // 0 = RGB
    // 1 = BGR
    // 2 = Bitmask
    // 3 = BltOnly
    pub framebuffer_format: u32,

    pub memory_map_addr: u64,
    pub memory_map_size: u64,
    pub memory_map_descriptor_size: u32,
    pub memory_map_descriptor_version: u32,
}