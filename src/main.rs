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
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(const_atomic_usize_new)]
#![feature(global_allocator, heap_api)]

#[macro_use]
mod vga_buffer;
mod interrupts;
mod memory;
mod features;

extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate os_bootinfo;
extern crate spin;
#[macro_use]
extern crate bitflags;
extern crate raw_cpuid;
extern crate x86_64;
#[macro_use]
extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate once;
extern crate linked_list_allocator;

use os_bootinfo::BootInfo;
use vga_buffer::Color;

//use memory::heap_allocator::BumpAllocator;

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

    let green = Color::Green;
    let blue = Color::Blue;

    vga_buffer::clear_screen();

    //vga_buffer::write_at("#", 10, 10, green);

    let mut x = 20;
    let mut y = 20;
    let mut x_old = x;
    let mut y_old = y;

    let cpuid = raw_cpuid::CpuId::new();
    println!("processor info {:?}", cpuid.get_processor_frequency_info());
    //    println!("hz {:?}", calc_cpu_freq());

    //interrupts::init();

    memory::init(boot_info);

    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    use alloc::boxed::Box;
    let mut heap_test = Box::new(42);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    let mut vec_test = vec![1,2,3,4,5,6,7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    let clock = features::clock::Clock::new();
    clock.uptime();

    loop {}
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

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
//
//#[global_allocator]
//static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);


use linked_list_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();