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
#[macro_use]
extern crate raw_cpuid;

use os_bootinfo::BootInfo;
use vga_buffer::Color;

use raw_cpuid::{ProcessorFrequencyInfo, CpuId};

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

    let cpuid = CpuId::new();
    let time = x86_64::instructions::rdtsc();
    println!("processor info {:?}", cpuid.get_processor_frequency_info());
    /*let time = x86_64::instructions::rdtsc();
    println!("processor info {:?}", time);
    let time = x86_64::instructions::rdtsc();
    println!("processor info {:?}", time);
    let time = x86_64::instructions::rdtsc();
    println!("processor info {:?}", time);*/


    /*loop {
        sleep();
        vga_buffer::write_at(" ", x_old, y_old, green);
        vga_buffer::write_at("#", x, y, green);
        y_old = y;
        x_old = x;
        y = (y + 1) % 20;
        x = (x + 1) % 20;
    }*/
    init_clock();
    uptime();

    loop {}
}

pub fn sleep() {
    //1342302694
    for i in 0..500_000{ let x = i;}
}

pub fn uptime() {
    let color = Color::Magenta;
    loop {
        match vga_buffer::read_at(0,78) {
            48 => vga_buffer::write_at("1", 0, 78, color),
            49 => vga_buffer::write_at("2", 0, 78, color),
            50 => vga_buffer::write_at("3", 0, 78, color),
            51 => vga_buffer::write_at("4", 0, 78, color),
            52 => vga_buffer::write_at("5", 0, 78, color),
            53 => vga_buffer::write_at("6", 0, 78, color),
            54 => vga_buffer::write_at("7", 0, 78, color),
            55 => vga_buffer::write_at("8", 0, 78, color),
            56 => vga_buffer::write_at("9", 0, 78, color),
            57 => {
                vga_buffer::write_at("0", 0, 78, color);
                increase_time(77,78, 2);},
            _ => vga_buffer::write_at("X", 0, 78, color),
        }
        sleep();
    }
    println!("vga read: {:?}", vga_buffer::read_at(0,74));
}

pub fn increase_time(col : u8, col_small: u8, step: u8) {
    let color = Color::Magenta;
    match vga_buffer::read_at(0,col as usize) {
        48 => vga_buffer::write_at("1", 0, col, color),
        49 => vga_buffer::write_at("2", 0, col, color),
        50 => vga_buffer::write_at("3", 0, col, color),
        51 => vga_buffer::write_at("4", 0, col, color),
        52 => vga_buffer::write_at("5", 0, col, color),
        53 => {
            let mut step = step;
            if vga_buffer::read_at(0,col as usize) == 58 { step = step +1}
            vga_buffer::write_at("0", 0, col, color);
            if col > 70 {increase_time(col - step,78, (step + 1));}},
        _ => vga_buffer::write_at("X", 0, col, color),
    }
}

pub fn init_clock() {
    let color = Color::Magenta;
    vga_buffer::write_at(":", 0, 76, color);
    vga_buffer::write_at(":", 0, 73, color);
    vga_buffer::write_at("0", 0, 71, color);
    vga_buffer::write_at("0", 0, 72, color);
    vga_buffer::write_at("0", 0, 74, color);
    vga_buffer::write_at("0", 0, 75, color);
    vga_buffer::write_at("0", 0, 77, color);
    vga_buffer::write_at("0", 0, 78, color);
}
