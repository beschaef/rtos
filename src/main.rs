//! This Operation System is based on the blog posts by phil oppermann.
//! While this os was programmed the second edition was not finished. So the system is using all
//! new stuff of the second edition which was ready. The rest which was needed are taken from the
//! first edition of the blog posts.
//!
//! There are still some very very rare bugs.
//! It can happen that the scheduler is called before the tasks are initialzed. This causes a panic
//! and the system dies.
//!
//! There are also very rarely `Page Faults` which are not clear where they came from. This is hard
//! to debug, because the `gdb` debugger will cause other faults when trying to debug.
//!
#![feature(lang_items)]
#![no_std]
#![no_main]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(alloc, collections)]
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

use alloc::string::{String, ToString};
use features::{active_sleep, get_cpu_freq, msleep, disable_cursor};
use os_bootinfo::BootInfo;
use raw_cpuid::CpuId;
use tasks::{tetris, uptime_temp};
use interrupts::fault_reboot;

/// Used when a panic occour. The function prints the file and the line on the screen when a panic
/// occur.
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
    fault_reboot();
    loop {}
}

/// This is the function for the entry point of the system.
/// Here gets the system initialized.
/// For a system with multiple tasks with a scheduler it is important have a memory_controller to
/// allocate stack for each task. To handle strings, Vec, etc. a heap allocator is needed. This
/// system is using a linked list heap alloctor. Actually its only possible to allocate heap and stack.
/// So it is important to reuse as much Variables as possible.
///
/// We decided to continue using the memory_controller in the main task, because this is not global
/// in the current version. Although it is possible to pass the memory_controller to a function, but
/// it is not possible to use it in a new task.
///
/// To start additional tasks in the running system, they must be added to the vector NEW_TASKS.
/// The main task looks every 200ms for new tasks in the vector. If there is a new task, it is
/// allocated for this new memory and then pushed to the TASKS vector, which the scheduler uses.
///
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    boot_info
        .check_version()
        .expect("Bootinfo version do not match");

    let mut memory_controller = memory::init(boot_info);

    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP_START, HEAP_START + HEAP_SIZE);
    }

    disable_cursor();

    // initialize our IDT
    interrupts::init(&mut memory_controller);
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    let freq = get_cpu_freq();

    let cpuid = CpuId::new();

    scheduler::sched_init(&mut memory_controller);

    let mut vendor_info = "".to_string();
    if let Some(info) = cpuid.get_vendor_info() {
        vendor_info = info.as_string().to_string();
        trace_fatal!("Vendor: {}\n", vendor_info);
    }

    let mut brand_info = "".to_string();
    if let Some(info) = cpuid.get_extended_function_info() {
        if let Some(brand) = info.processor_brand_string() {
            brand_info = brand.to_string();
            trace_fatal!("Model: {}\n", brand_info);
        }
    }

    print_welcome(vendor_info, brand_info);
    print_booting();

    interrupts::init_timer();
    msleep(1000);

    trace_fatal!("Freq{:?}", cpuid!(1));
    trace_fatal!("System Info");
    trace_fatal!("Calculated CPU-frequency: {}", freq);
    trace_fatal!("Heap Size: {}", HEAP_SIZE);
    set_trace_level!(TraceLevel::Debug);

    loop {
        msleep(200);
        if let Some(f) = tasks::NEW_TASKS.lock().pop() {
            let mut name: char;
            if f == x86_64::VirtualAddress(tetris as usize) {
                name = 't';
            } else if f == x86_64::VirtualAddress(uptime_temp as usize) {
                name = 'u';
            } else {
                name = 'm';
            }
            trace_warn!("added new task");
            let memory = memory_controller
                .alloc_stack(4)
                .expect("can't allocate stack");
            scheduler::TASKS.lock().push(tasks::TaskData::new(
                name,
                0,
                x86_64::VirtualAddress(memory.top()),
                f,
                tasks::TaskStatus::READY,
            ));
        }
    }
}

pub const HEAP_START: usize = 0o_000_001_000_000_0000;
pub const HEAP_SIZE: usize = 300 * 1024; // 100 KiB
                                         //
                                         //#[global_allocator]
                                         //static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);

use linked_list_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// prints a welcome message on the screen
fn print_welcome(vendor_info: String, brand_info: String) {
    println!(" #     #                                                           ");
    println!(" #  #  # ###### #       ####   ####  #    # ######    #####  ####  ");
    println!(" #  #  # #      #      #    # #    # ##  ## #           #   #    # ");
    println!(" #  #  # #####  #      #      #    # # ## # #####       #   #    #  ");
    println!(" #  #  # #      #      #      #    # #    # #           #   #    #  ");
    println!(" #  #  # #      #      #    # #    # #    # #           #   #    #   ");
    println!("  ## ##  ###### ######  ####   ####  #    # ######      #    #### ");
    println!("                                                                     ");
    println!("                      ######  ####### #######  #####         ");
    println!("                      #     #    #    #     # #     #         ");
    println!("                      #     #    #    #     # #           ");
    println!("                      ######     #    #     #  #####     ");
    println!("                      #   #      #    #     #       #    ");
    println!("                      #    #     #    #     # #     #   ");
    println!("                      #     #    #    #######  #####     ");
    println!("");
    println!("");
    println!("");
    println!("{}", vendor_info);
    println!("{}", brand_info);

    active_sleep(3500);
    for _x in 0..vga_buffer::BUFFER_HEIGHT / 4 {
        println!("");
    }
}

/// prints booting information to the screen
fn print_booting() {
    println!(" #######                   #####");
    println!("    #    #    # ######    #     # #   #  ####  ##### ###### #    #");
    println!("    #    #    # #         #        # #  #        #   #      ##  ##");
    println!("    #    ###### #####      #####    #    ####    #   #####  # ## #");
    println!("    #    #    # #               #   #        #   #   #      #    #");
    println!("    #    #    # #         #     #   #   #    #   #   #      #    #");
    println!("    #    #    # ######     #####    #    ####    #   ###### #    #");
    println!("");
    println!("");
    println!(" #  ####     #####   ####   ####  ##### # #    #  ####");
    println!(" # #         #    # #    # #    #   #   # ##   # #    #");
    println!(" #  ####     #####  #    # #    #   #   # # #  # #");
    println!(" #      #    #    # #    # #    #   #   # #  # # #  ###");
    println!(" # #    #    #    # #    # #    #   #   # #   ## #    #");
    println!(" #  ####     #####   ####   ####    #   # #    #  ####");
    println!("");
    println!("");
    println!("#####  #      ######   ##    ####  ######    #    #   ##   # #####");
    println!("#    # #      #       #  #  #      #         #    #  #  #  #   #");
    println!("#    # #      #####  #    #  ####  #####     #    # #    # #   #");
    println!("#####  #      #      ######      # #         # ## # ###### #   #");
    println!("#      #      #      #    # #    # #         ##  ## #    # #   #");
    println!("#      ###### ###### #    #  ####  ######    #    # #    # #   #");
    active_sleep(1000);
    for _x in 0..vga_buffer::BUFFER_HEIGHT {
        println!("");
    }
}
