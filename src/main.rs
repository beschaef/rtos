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
#![feature(naked_functions)]
#![feature(core_intrinsics)]

#[macro_use]
mod vga_buffer;
#[macro_use]
mod trace;
mod features;
mod interrupts;
mod memory;
mod pic;
mod scheduler;
mod tasks;

extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate os_bootinfo;
extern crate spin;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate raw_cpuid;
#[macro_use]
extern crate x86_64;
#[macro_use]
extern crate alloc;
extern crate rlibc;
#[macro_use]
extern crate once;
extern crate bit_field;
extern crate cpuio;
extern crate linked_list_allocator;

use features::{get_cpu_freq, msleep};
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

    let mut memory_controller = memory::init(boot_info);

    //vga_buffer::clear_screen();

    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    // initialize our IDT
    interrupts::init(&mut memory_controller);
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    let freq = get_cpu_freq();
    let cpuidss = raw_cpuid::CpuId::new();
    vga_buffer::write_at_background(&format!("{}",freq), 22, 0, Color::Magenta, Color::Black);
    trace_fatal!();
    trace_fatal!("CPU INFO {:?}", cpuid!(0x16));
    trace_fatal!("CPU INFO {:?}", cpuid!(0x16,0));
    trace_fatal!("System Info");
    trace_fatal!("Calculated CPU-frequency: {}", freq);
    trace_fatal!("Heap Size: {}", HEAP_SIZE);
    trace_debug!();
    trace_info!();
    trace_info!();
    set_trace_level!(TraceLevel::Debug);
    trace_info!();
    trace_warn!();
    trace_error!();

    scheduler::sched_init(&mut memory_controller);
    interrupts::init_timer();
    msleep(1000);
    loop {
        msleep(200);
        if let Some(f) = tasks::NEW_TASKS.lock().pop() {
            trace_warn!("added new task");
            let memory = memory_controller
                .alloc_stack(2)
                .expect("can't allocate stack");
            scheduler::TASKS.lock().push(tasks::TaskData::new(
                0,
                x86_64::VirtualAddress(memory.top()),
                f,
                tasks::TaskStatus::READY,
            ));
        }
    }
}

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 200 * 1024; // 100 KiB
                                         //
                                         //#[global_allocator]
                                         //static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);

use linked_list_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
