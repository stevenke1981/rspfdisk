//! Host builds a stub; UEFI target builds BOOTX64.EFI.

#![cfg_attr(target_os = "uefi", no_main)]
#![cfg_attr(target_os = "uefi", no_std)]

#[cfg(target_os = "uefi")]
extern crate alloc;

#[cfg(target_os = "uefi")]
use uefi::allocator::Allocator;

#[cfg(target_os = "uefi")]
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

#[cfg(target_os = "uefi")]
mod uefi_entry;

#[cfg(target_os = "uefi")]
use uefi::prelude::*;

#[cfg(target_os = "uefi")]
#[entry]
fn main() -> Status {
    uefi_entry::run()
}

#[cfg(not(target_os = "uefi"))]
fn main() {
    eprintln!("BOOTX64.EFI: build with --target x86_64-unknown-uefi");
    eprintln!("  cargo build -p rspfdisk-uefi --release --target x86_64-unknown-uefi");
}
