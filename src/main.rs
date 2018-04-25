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
#![feature(ptr_internals)]

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
extern crate x86_64;

use os_bootinfo::BootInfo;
use vga_buffer::Color;

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

    for (i,ele) in boot_info.memory_map.iter().enumerate() {
        let region_type = ele.region_type;
        let start_address = ele.range.start_addr();
        let end_address = ele.range.end_addr();
        let size = end_address - start_address;
        println!(
            "{:<2}: start: 0x{:<10x} end: 0x{:<10x} size: 0x{:<8x} {:?}",
            i, start_address, end_address, size, region_type,
        );
    }
    let mut frame_allocator = memory::AreaFrameAllocator::new(
        &boot_info.memory_map,
    );

    memory::test_paging(&mut frame_allocator);

    println!("{:?}", frame_allocator.allocate_frame());
    println!("{:?}", frame_allocator.allocate_frame());
    println!("{:?}", frame_allocator.allocate_frame());

    for i in 0.. {
        if let None = frame_allocator.allocate_frame() {
            println!("allocated {} frames", i); // printed 31978 frames
                                                // sind 31593 aus der 2ten usable region +
                                                // 385 aus der ersten, die erste hätte 390
                                                // wir haben aber aufgrund test_paging +
                                                // die 3 prints schon 5 verbraucht
            break;
        }
    }

    println!("{:?}", frame_allocator.allocate_frame());
    println!("{:?}", frame_allocator.allocate_frame());
    println!("{:?}", frame_allocator.allocate_frame());
    println!("{:?}", frame_allocator.allocate_frame());

    let green = Color::Green;
    let blue = Color::Blue;

    vga_buffer::clear_screen();

    //vga_buffer::write_at("#", 10, 10, green);

    let mut x = 20;
    let mut y = 20;
    let mut x_old = x;
    let mut y_old = y;
    loop {
        sleep();
        vga_buffer::write_at(" ", x_old, y_old, green);
        vga_buffer::write_at("#", x, y, green);
        y_old = y;
        x_old = x;
        y = (y + 1) % 30;
    }

    loop {}
}

pub fn sleep() {
    for i in 0..500000{ let x = i;}
}
