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

use features::{active_sleep, get_cpu_freq, msleep};
use os_bootinfo::BootInfo;
use raw_cpuid::CpuId;
use vga_buffer::Color;

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

    let cpuid = CpuId::new();

    //vga_buffer::write_at_background(cpuid.max_eax_value, 19,2, Color::Blue, Color::Red);

    //msleep(10000);

    scheduler::sched_init(&mut memory_controller);

    print_welcome();
    print_booting();

    interrupts::init_timer();
    msleep(1000);

    if let Some(info) = cpuid.get_vendor_info() {
        vga_buffer::write_at_background(info.as_string(), 20, 2, Color::Blue, Color::Red);
        trace_fatal!("Vendor: {}\n", info.as_string());
    }

    if let Some(info) = cpuid.get_extended_function_info() {
        if let Some(brand) = info.processor_brand_string() {
            vga_buffer::write_at_background(brand, 21, 2, Color::Blue, Color::Red);
            trace_fatal!("Model: {}\n", brand);
        }
    }

    if let Some(info) = cpuid.get_processor_frequency_info() {
        //println!("CPU Base MHz: {}\n", info.processor_base_frequency());
        //println!("CPU Max MHz: {}\n", info.processor_max_frequency());
        //println!("Bus MHz: {}\n", info.bus_frequency());
        trace_fatal!("CPU Base MHz: {}\n", info.processor_base_frequency());
        trace_fatal!("CPU Max MHz: {}\n", info.processor_max_frequency());
        trace_fatal!("Bus MHz: {}\n", info.bus_frequency());
        vga_buffer::write_at_background(
            &format!("CPU Base MHz: {}\n", info.processor_base_frequency()),
            22,
            2,
            Color::Blue,
            Color::Red,
        );
        vga_buffer::write_at_background(
            &format!("CPU Max MHz: {}\n", info.processor_max_frequency()),
            23,
            2,
            Color::Blue,
            Color::Red,
        );
    } else {
        vga_buffer::write_at_background("Can't get cpu freq info!", 23, 2, Color::Blue, Color::Red);
    }

    trace_fatal!("Freq{:?}", cpuid!(1));
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

    loop {
        msleep(200);
        if let Some(f) = tasks::NEW_TASKS.lock().pop() {
            trace_warn!("added new task");
            let memory = memory_controller
                .alloc_stack(3)
                .expect("can't allocate stack");
            scheduler::TASKS.lock().push(tasks::TaskData::new(
                'm',
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

/// prints a welcome message on the screen
fn print_welcome() {
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
    active_sleep(1000);
    for x in 0..vga_buffer::BUFFER_HEIGHT / 4 {
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
    for x in 0..vga_buffer::BUFFER_HEIGHT {
        println!("");
    }
}
