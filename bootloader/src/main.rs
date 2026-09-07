#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    uefi::println!();
    uefi::println!("================================");
    uefi::println!("       MENTACORE BOOTLOADER");
    uefi::println!("================================");
    uefi::println!();
    uefi::println!("UEFI initialized.");
    uefi::println!("Bootloader entry reached.");
    uefi::println!();
    uefi::println!("MENTACORE BOOTLOADER OK");
    uefi::println!();

    loop {
        core::hint::spin_loop();
    }
}