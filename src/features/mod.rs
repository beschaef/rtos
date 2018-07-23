pub mod clock;
pub mod keyboard;

use scheduler::RUNNING_TASK;
use x86_64;
use x86_64::instructions::rdtsc;

/// global variable to store the cpu frequency
static mut CPU_FREQ: u64 = 0;

/// With pit and tsc the cpu frequency is calculated. To prevent interrupt issues, all intterrupts are
/// disabled during the calulation. Currently there is no `rust-way` to initialize and modify the pit. Therefore
/// the function is using inline assembly to initialize the pit.
/// To calculate the frequency the current pit and tsc are read two times with a minimum difference of 40 pit ticks.
/// After that the difference of both timer is taken and then the quotient is taken.
/// For a better estimate the steps are repeated 5 times.
/// The last step is to multiply the result value with the pit frequency(1.193MHz) to get the cpu frequency.
pub fn calc_freq() -> u64 {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    let mut r0: u64 = 0;
    let mut f0: u64 = 0;
    let mut r1: u64 = 0;
    let mut r2: u64 = 0;
    let mut t0: u64 = 0;
    let mut t1: u64 = 0;
    let mut hi: u64 = 0;
    let mut lo: u64 = 0;
    let mut ticks: u64 = 0;
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

    for i in 0..5 {
        t0 = read_pit();
        t1 = t0;
        trace_fatal!("t0 {:?}", t0);
        while (t0 - t1) < 20 {
            t1 = read_pit();
            unsafe {
                trace_fatal!("t1 {:?}", t1);
                r1 = x86_64::instructions::rdtsc();
            }
        }
        t0 = t1;
        while (t0 - t1) < 40 {
            t1 = read_pit();
            trace_fatal!("t1 {:?}", t1);
            unsafe {
                r2 = x86_64::instructions::rdtsc();
            }
        }
        r0 += r2 - r1;
        trace_fatal!("r0 {:?}", r0);

        f0 += (t0 - t1);
        trace_fatal!("f0 {:?}", f0);
    }
    trace_fatal!("freq {:?}", r0 / f0);
    trace_fatal!("freq {:?}", (r0 / f0 * pit_freq));

    return (r0 / f0 * pit_freq) as u64;
}

/// Currently there is no `rust-way` to get the remaining pit-ticks. Therefore the function use
/// inline assembly to get the remaining pit-ticks.
fn read_pit() -> u64 {
    let mut reg: u64 = 0;
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

/// This function returns the cpu frequency, if frequency is unknown, the cpu frequency is calculated.
pub fn get_cpu_freq() -> u64 {
    unsafe {
        if CPU_FREQ == 0 {
            CPU_FREQ = calc_freq();
        }
        CPU_FREQ as u64
    }
}

/// The function calculates how many tsc ticks the current process has to sleep in dependent on the
/// given time in milliseconds. After this the function saves the `sleep_ticks` in the `RUNNING_TASK`
/// struct. To prevent CPU wasting the timer interrupt is called an thus the scheduler is called.
pub fn msleep(ms: u64) {
    trace_info!();
    let one_sec = get_cpu_freq();

    let time = one_sec * (ms / 1000); // (one_sec * ms / 1000) as i64; doesnt work!
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
