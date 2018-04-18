//! Code for the RTOS
//!
//! #![no_std] is used to disable the standard library
//! #![no_main] is added to tell the rust compiler that we don't want to use
//! the normal entry point chain. This also requires to remove the main
//! function, because there's nothing to call the main
//!
//! to build and link the Code on Linux use
//! 'cargo rustc -- -Z pre-link-arg=-nostartfiles'
//!
//! to build and link it under macOS use
//! 'cargo rustc -- -Z pre-link-arg=-lSystem'
//!
//! to build our programm without an underlaying OS use
//! 'xargo build --target x86_64-rtos
//!
#![feature(lang_items)]
#![no_std]
#![no_main]
#![feature(const_fn)]

#[macro_use]
mod vga_buffer;
mod memory;

extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate os_bootinfo;
extern crate spin;
#[macro_use]
extern crate bitflags;

use os_bootinfo::BootInfo;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
    msg: core::fmt::Arguments,
    file: &'static str,
    line: u32,
    _column: u32,
) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", msg);
    loop {}
}

// this is the function for the entry point on Linux.
// the "-> !"" means that the function is diverging, i.e. not allowed to ever return.
// 0xb8000 is the address of the VGA buffer
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    boot_info
        .check_version()
        .expect("Bootinfo version do not match");
    use memory::FrameAllocator;

    for ele in boot_info.memory_map.iter() {
        let region_type = ele.region_type;
        let start_address = ele.range.start_addr();
        let end_address = ele.range.end_addr();
        let size = end_address - start_address;
        println!(
            "{:x?} start: {:x?} end: {:x?} size: {:x?}",
            region_type,
            start_address,
            end_address,
            size,
        );
    }

    let kernel_start = boot_info.memory_map[3].range.start_addr();
    let kernel_end = boot_info.memory_map[3].range.end_addr();
    let bootloader_start = boot_info.memory_map[2].range.start_addr();
    let bootloader_end = boot_info.memory_map[2].range.end_addr();

    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize, bootloader_start as usize,
        bootloader_end as usize, &boot_info.memory_map);

    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i);
            break;
        }
    }
    //println!("{:?}", frame_allocator.allocate_frame());

    loop {}
}
