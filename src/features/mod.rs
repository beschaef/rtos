//! Provides different helper functions which can't be assigned to a specific module.
pub mod clock;
pub mod keyboard;
pub mod shell;

use raw_cpuid::CpuId;
use scheduler::RUNNING_TASK;
use x86_64;
use x86_64::instructions::{port, rdtsc};

/// Global variable to store the cpu frequency.
static mut CPU_FREQ: u64 = 0;

/// With pit and tsc the cpu frequency is calculated. To prevent interrupt issues, all interrupts are
/// disabled during the calulation. Currently there is no *rust-way* to initialize and modify the pit. Therefore
/// the function is using inline assembly to initialize it.
/// To calculate the frequency, the current pit and tsc are read two times with a minimum difference of 40 pit ticks.
/// After that the difference of both timers is taken and then the quotient is calculated.
/// For a better estimatation the steps are repeated five times.
/// The last step is to multiply the result value with the pit frequency(1.193 MHz) to get the cpu frequency.
pub fn calc_freq() -> u64 {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    let mut r0: u64 = 0;
    let mut f0: u64 = 0;
    let mut r1: u64 = 0;
    let mut r2: u64 = 0;
    let mut t0: u64;
    let mut t1: u64;
    let pit_freq = 1193182;

    trace_fatal!("pit_freq {:?}", pit_freq as u64);

    unsafe {
        asm!("
                    mov al, 0x34
                    out 0x43, al

                    mov rcx, 50000
                    mov al, cl
                    out 0x40, al
                    mov al, ch
                    out 0x40, al"::::"intel", "volatile");
    }

    for _i in 0..5 {
        t0 = read_pit();
        t1 = t0;
        trace_fatal!("t0 {:?}", t0);
        while (t0 - t1) < 20 {
            t1 = read_pit();
            trace_fatal!("t1 {:?}", t1);
            r1 = x86_64::instructions::rdtsc();
        }
        t0 = t1;
        while (t0 - t1) < 40 {
            t1 = read_pit();
            trace_fatal!("t1 {:?}", t1);
            r2 = x86_64::instructions::rdtsc();
        }
        r0 += r2 - r1;
        trace_fatal!("r0 {:?}", r0);

        f0 += t0 - t1;
        trace_fatal!("f0 {:?}", f0);
    }
    trace_fatal!("freq {:?}", r0 / f0);
    trace_fatal!("freq {:?}", (r0 / f0 * pit_freq));

    return (r0 / f0 * pit_freq) as u64;
}

/// Currently there is no *rust-way* to get the remaining pit ticks. Therefore the function uses
/// inline assembly to get the remaining pit ticks.
fn read_pit() -> u64 {
    let reg: u64;
    unsafe {
        asm!("   and rax, 0
                 mov     al,0h
                 out     43h,al
                 in      al,40h
                 mov     ah,al
                 in      al,40h
                 rol     ax,8

                 push rax
                 pop $0
                 ":"=r"(reg)::"rax":"intel","volatile");
    }
    return reg;
}

/// There are three ways to get the CPU frequency in a bare-bone system.
/// This function tries all three ways and takes the first which is working.
///
/// The first way is to get it directly out of the registry. For this there is the crate `raw_cpuid`
/// which provides a lot of functions. In this case the two functions `get_processor_frequency_info()`
/// and `processor_base_frequency()` are used to get the frequency. This is not working in all
/// ways, i.e. in qemu the function returns 0. On the other hand, when the system is loaded on an
/// usb stick and booted directly, the function returned the frequency on all tested systems.
///
/// The second way also uses the crate`raw_cpuid`. Most systems are returning their processor
/// brand when using the function `processor_brand_string()`.
/// The string includes the frequency in GHz. For example, one tested computer returned
/// `Intel(R) Core(TM) i3-4010U CPU @ 1.70GHz`. To get the frequency the function is
/// looking for the substring ` @ ` and then converts the following 'string numbers' into numbers.
///
/// If the processor brand also is empty, it is possible to calculate / approximate the frequency.
/// This approximation is never really precise, so this is the last way to get the frequency. The calculation is described in
/// `features::calc_freq()`
pub fn get_cpu_freq() -> u64 {
    let cpuid = CpuId::new();
    if let Some(info) = cpuid.get_processor_frequency_info() {
        unsafe {
            CPU_FREQ = info.processor_base_frequency() as u64 * 1024 * 1024;
        }
    }
    if unsafe { CPU_FREQ } == 0 {
        if let Some(info) = cpuid.get_extended_function_info() {
            if let Some(brand) = info.processor_brand_string() {
                trace_fatal!("tes{:?}", "t");
                let mut first: char;
                let mut second = 'a';
                let mut third = 'a';
                let mut found_freq = false;
                let mut found_dot = false;
                let mut digit_big = 0.0;
                let mut digit_small = 0.0;
                let mut step = 0.1;
                for b in brand.chars() {
                    first = second;
                    second = third;
                    third = b;
                    if first == ' ' && second == '@' && third == ' ' {
                        found_freq = true;
                    }
                    if found_freq && b.is_numeric() && !found_dot {
                        digit_big = (digit_big * 10.0) + b.to_digit(10).unwrap() as f32;
                    }
                    if found_freq && b == '.' {
                        found_dot = true;
                        continue;
                    }
                    if found_dot && b.is_numeric() {
                        digit_small = digit_small + b.to_digit(10).unwrap() as f32 * step;
                        step *= 0.1;
                    }
                    if found_freq && found_dot && !b.is_numeric() {
                        break;
                    }
                }
                if found_freq {
                    unsafe {
                        CPU_FREQ = ((digit_big + digit_small) * 1000.0 * 1000.0 * 1000.0) as u64
                    };
                }
            }
        }
    }
    unsafe {
        if CPU_FREQ == 0 {
            CPU_FREQ = calc_freq();
        }
        CPU_FREQ as u64
    }
}

/// The function calculates how many tsc ticks the current process has to sleep, dependent on the
/// given time in milliseconds. After this the function saves the `sleep_ticks` in the `RUNNING_TASK`
/// struct. To prevent CPU waste, the timer interrupt is called and thus the scheduler is called.
pub fn msleep(ms: u64) {
    trace_info!();
    let one_sec = get_cpu_freq();

    let time = one_sec * ms / 1000; // (one_sec * ms / 1000) as i64; doesnt work!
    let tsc = time + rdtsc();
    trace_debug!("sleep until: {}", tsc);
    unsafe {
        {
            x86_64::instructions::interrupts::disable();
            RUNNING_TASK.lock().sleep_ticks = tsc as usize;
            x86_64::instructions::interrupts::enable();
        }
        int!(0x20);
    }
}

/// This sleep is not calling the scheduler.
/// It is used for early sleeps, before any tasks oder scheduler are running.
pub fn active_sleep(ms: u64) {
    let one_sec = get_cpu_freq();
    let time = one_sec * ms / 1000; // (one_sec * ms / 1000) as i64; does'nt work!
    let tsc = time + rdtsc();
    let mut wait = rdtsc();
    while wait < tsc {
        wait = rdtsc();
    }
}

/// Causes the system to reboot.
/// Based on [OSDev Wiki](https://wiki.osdev.org/Reboot).
pub fn reboot() {
    let mut good = 0x02;
    while good & 0x02 == 1 {
        unsafe {
            good = port::inb(0x64);
        }
    }
    unsafe {
        port::outb(0x64, 0xFE);
    }
}

/// Causes the system to shutdown.
/// Based on [OSDev Wiki](https://wiki.osdev.org/Reboot).
pub fn shutdown() {
    unsafe { port::outb(0xf4, 0x00) };
}

/// Tests if a specific bit is set in a byte
pub fn test_bit(byte: u8, bit: u8) -> bool {
    byte & bit > 0
}

/// Disables the cursor on the vga_buffer.
/// More information at [OSDev Wiki](https://wiki.osdev.org/Text_Mode_Cursor#Disabling_the_Cursor).
pub fn disable_cursor() {
    unsafe {
        port::outb(0x3D4, 0x0A);
        port::outb(0x3D5, 20);
    }
}
