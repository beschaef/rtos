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
#![feature(asm)]
#![feature(abi_x86_interrupt)]

#[macro_use]
mod vga_buffer;
mod interrupts;
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

use raw_cpuid::{CpuId, ProcessorFrequencyInfo};

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

    /*for (i, ele) in boot_info.memory_map.iter().enumerate() {
        let region_type = ele.region_type;
        let start_address = ele.range.start_addr();
        let end_address = ele.range.end_addr();
        let size = end_address - start_address;
        println!(
            "{:<2}: start: 0x{:<10x} end: 0x{:<10x} size: 0x{:<8x} {:?}",
            i, start_address, end_address, size, region_type,
        );
    }*/

    let mut frame_allocator = memory::AreaFrameAllocator::new(&boot_info.memory_map);

    let green = Color::Green;
    let blue = Color::Blue;

    vga_buffer::clear_screen();

    //vga_buffer::write_at("#", 10, 10, green);

    let mut x = 20;
    let mut y = 20;
    let mut x_old = x;
    let mut y_old = y;

    println!("processor info {:?}", cpuid.get_processor_frequency_info());
    println!("hz {:?}", calc_cpu_freq());

    interrupts::init();

    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();

    println!("It did not crash!");
    loop {}

    init_clock();
    uptime();

    loop {}
    
}

pub fn sleep() {
    //1342302694
    for i in 0..500_000 {
        let x = i;
    }
}

pub fn uptime() {
    let color = Color::LightGreen;
    loop {
        match vga_buffer::read_at(0, 78) {
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
                match vga_buffer::read_at(0, 77 as usize) {
                    48 => vga_buffer::write_at("1", 0, 77, color),
                    49 => vga_buffer::write_at("2", 0, 77, color),
                    50 => vga_buffer::write_at("3", 0, 77, color),
                    51 => vga_buffer::write_at("4", 0, 77, color),
                    52 => vga_buffer::write_at("5", 0, 77, color),
                    53 => {
                        vga_buffer::write_at("0", 0, 77, color);
                        increase_minute();
                    }
                    _ => vga_buffer::write_at("X", 0, 77, color),
                }
            }
            _ => vga_buffer::write_at("X", 0, 78, color),
        }
        sleep();
    }
    println!("vga read: {:?}", vga_buffer::read_at(0, 74));
}

pub fn increase_minute() {
    let color = Color::LightGreen;
    match vga_buffer::read_at(0, 75) {
        48 => vga_buffer::write_at("1", 0, 75, color),
        49 => vga_buffer::write_at("2", 0, 75, color),
        50 => vga_buffer::write_at("3", 0, 75, color),
        51 => vga_buffer::write_at("4", 0, 75, color),
        52 => vga_buffer::write_at("5", 0, 75, color),
        53 => vga_buffer::write_at("6", 0, 75, color),
        54 => vga_buffer::write_at("7", 0, 75, color),
        55 => vga_buffer::write_at("8", 0, 75, color),
        56 => vga_buffer::write_at("9", 0, 75, color),
        57 => {
            vga_buffer::write_at("0", 0, 75, color);
            match vga_buffer::read_at(0, 74 as usize) {
                48 => vga_buffer::write_at("1", 0, 74, color),
                49 => vga_buffer::write_at("2", 0, 74, color),
                50 => vga_buffer::write_at("3", 0, 74, color),
                51 => vga_buffer::write_at("4", 0, 74, color),
                52 => vga_buffer::write_at("5", 0, 74, color),
                53 => {
                    vga_buffer::write_at("0", 0, 74, color);
                    increase_hour();
                }
                _ => vga_buffer::write_at("X", 0, 74, color),
            }
        }
        _ => vga_buffer::write_at("X", 0, 75, color),
    }
}

pub fn increase_hour() {
    let color = Color::LightGreen;
    match (vga_buffer::read_at(0, 71), vga_buffer::read_at(0, 72)) {
        (48, 48) => vga_buffer::write_at("1", 0, 72, color),
        (48, 49) => vga_buffer::write_at("2", 0, 72, color),
        (48, 50) => vga_buffer::write_at("3", 0, 72, color),
        (48, 51) => vga_buffer::write_at("4", 0, 72, color),
        (48, 52) => vga_buffer::write_at("5", 0, 72, color),
        (48, 53) => vga_buffer::write_at("6", 0, 72, color),
        (48, 54) => vga_buffer::write_at("7", 0, 72, color),
        (48, 55) => vga_buffer::write_at("8", 0, 72, color),
        (48, 56) => vga_buffer::write_at("9", 0, 72, color),
        (48, 57) => {
            vga_buffer::write_at("0", 0, 72, color);
            vga_buffer::write_at("1", 0, 71, color);
        }
        (49, 48) => vga_buffer::write_at("1", 0, 72, color),
        (49, 49) => vga_buffer::write_at("2", 0, 72, color),
        (49, 50) => vga_buffer::write_at("3", 0, 72, color),
        (49, 51) => vga_buffer::write_at("4", 0, 72, color),
        (49, 52) => vga_buffer::write_at("5", 0, 72, color),
        (49, 53) => vga_buffer::write_at("6", 0, 72, color),
        (49, 54) => vga_buffer::write_at("7", 0, 72, color),
        (49, 55) => vga_buffer::write_at("8", 0, 72, color),
        (49, 56) => vga_buffer::write_at("9", 0, 72, color),
        (49, 57) => {
            vga_buffer::write_at("0", 0, 72, color);
            vga_buffer::write_at("2", 0, 71, color);
        }
        (50, 51) => {
            vga_buffer::write_at("0", 0, 72, color);
            vga_buffer::write_at("0", 0, 71, color);
        }
        (50, 48) => vga_buffer::write_at("1", 0, 72, color),
        (50, 49) => vga_buffer::write_at("2", 0, 72, color),
        (50, 50) => vga_buffer::write_at("3", 0, 72, color),
        _ => vga_buffer::write_at("X", 0, 72, color),
    }
}

pub fn init_clock() {
    let color = Color::LightGreen;
    vga_buffer::write_at(":", 0, 76, color);
    vga_buffer::write_at(":", 0, 73, color);
    vga_buffer::write_at("0", 0, 71, color);
    vga_buffer::write_at("0", 0, 72, color);
    vga_buffer::write_at("0", 0, 74, color);
    vga_buffer::write_at("0", 0, 75, color);
    vga_buffer::write_at("0", 0, 77, color);
    vga_buffer::write_at("0", 0, 78, color);
}

pub fn calc_cpu_freq() -> usize {
    unsafe {
        /// I 59659 =  20hz
        const SIZE: usize = 10;
        let mut i = SIZE;
        let mut array: [usize; SIZE] = [0; SIZE];

        loop {
            i -= 1;
            asm!("

            mov  al,34h
            out  43h,al

            nop
            nop

            mov  rcx,65000

            mov  al,cl
            out  40h,al
            nop
            nop
            mov  al,ch
            out  40h,al
            nop
            nop"
        :::: "intel","volatile");

            let pit0: usize;
            let pit1: usize;
            let tsc0h: usize;
            let tsc0l: usize;
            let tsc1h: usize;
            let tsc1l: usize;
            asm!("

            and rax, 0
            mov     al,0h
            out     43h,al
            in      al,40h
            mov     ah,al
            in      al,40h
            rol     ax,8

            push rax
            pop $0
            rdtsc

            push rax
            pop $1
            push rdx
            pop $2

        loop:
            pause
            and rax, 0
            mov     al,0h
            out     43h,al
            in      al,40h
            mov     ah,al
            in      al,40h
            rol     ax,8
            cmp rax, 5000
            jge loop

            and rax, 0
            mov     al,0h
            out     43h,al
            in      al,40h
            mov     ah,al
            in      al,40h
            rol     ax,8

            push rax
            pop $3

            rdtsc
            mov    $4,rax
            mov    $5,rdx

            ":"=r"(pit0),"=r"(tsc0l),"=r"(tsc0h),"=r"(pit1),"=r"(tsc1l), "=r"(tsc1h)::"rax", "rdx", "rbx":"intel","volatile");

            //~ continue;
            if pit1 >= pit0 {
                print!("Pit0 {}, hex--> {0:X}", pit0);
                println!("                  Pit1 {}, hex--> {0:X} Count {}", pit1, i);
                i += 1;
                continue;
            }

            let tsc0 = tsc0h << 32 | tsc0l;
            let tsc1 = tsc1h << 32 | tsc1l;

            if tsc0 >= tsc1 {
                print!("TSC0 {}", tsc0);
                println!("     tsc1 {} Count {}", tsc1, i);
                i += 1;
                continue;
            }

            let diff_pit = pit0 - pit1;

            let diff_tsc = tsc1 - tsc0;
            let precision = 1000_000_000;
            let time_in_pit = (diff_pit * precision) / 1193181;

            let hz = (diff_tsc * precision) / time_in_pit;
            array[i] = hz;
            if i == 0 {
                break;
            }
        }
        //array.sort();
        let median: usize = array.len() / 2;
        let cpu_freq = array[median];
        return cpu_freq;
    }
}
