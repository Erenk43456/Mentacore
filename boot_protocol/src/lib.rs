#![no_std]

pub const BOOT_PROTOCOL_VERSION: u32 = 2;

pub const FRAMEBUFFER_FORMAT_RGB: u32 = 0;
pub const FRAMEBUFFER_FORMAT_BGR: u32 = 1;
pub const FRAMEBUFFER_FORMAT_BITMASK: u32 = 2;
pub const FRAMEBUFFER_FORMAT_BLT_ONLY: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BootInfo {
    pub version: u32,

    pub framebuffer_addr: u64,
    pub framebuffer_size: u64,

    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_stride: u32,
    pub framebuffer_format: u32,

    pub memory_map_addr: u64,
    pub memory_map_size: u64,
    pub memory_map_descriptor_size: u32,
    pub memory_map_descriptor_version: u32,

    // Kernel ELF image loaded by the bootloader.
    pub kernel_image_addr: u64,
    pub kernel_image_size: u64,

    // Userspace ELF image loaded by the bootloader.
    pub userspace_image_addr: u64,
    pub userspace_image_size: u64,
}