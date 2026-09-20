use mentacore_boot_protocol::{
    BootInfo,
    BOOT_PROTOCOL_VERSION,
    FRAMEBUFFER_FORMAT_BGR,
    FRAMEBUFFER_FORMAT_BITMASK,
    FRAMEBUFFER_FORMAT_BLT_ONLY,
    FRAMEBUFFER_FORMAT_RGB,
};

use uefi::boot;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::println;

const TARGET_WIDTH: usize = 1920;
const TARGET_HEIGHT: usize = 1080;

pub fn initialize() -> Result<BootInfo, ()> {
    let handle = boot::get_handle_for_protocol::<GraphicsOutput>()
        .map_err(|_| ())?;

    let mut gop =
        boot::open_protocol_exclusive::<GraphicsOutput>(handle)
            .map_err(|_| ())?;

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
        PixelFormat::Rgb => FRAMEBUFFER_FORMAT_RGB,
        PixelFormat::Bgr => FRAMEBUFFER_FORMAT_BGR,
        PixelFormat::Bitmask => FRAMEBUFFER_FORMAT_BITMASK,
        PixelFormat::BltOnly => FRAMEBUFFER_FORMAT_BLT_ONLY,
    };

    Ok(BootInfo {
        version: BOOT_PROTOCOL_VERSION,

        framebuffer_addr,
        framebuffer_size,
        framebuffer_width: width as u32,
        framebuffer_height: height as u32,
        framebuffer_stride: stride as u32,
        framebuffer_format,

        kernel_image_addr: 0,
        kernel_image_size: 0,

        userspace_image_addr: 0,
        userspace_image_size: 0,

        memory_map_addr: 0,
        memory_map_size: 0,
        memory_map_descriptor_size: 0,
        memory_map_descriptor_version: 0,
    })
}