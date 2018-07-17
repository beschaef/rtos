pub mod clock;
pub mod keyboard;

use scheduler::RUNNING_TASK;
use x86_64;
use x86_64::instructions::rdtsc;

static mut CPU_FREQ: u64 = 0;

/// Wird genutzt um die cpu_frequenz zu berechnen.
/// Code ist angelehnt an https://wiki.osdev.org/Detecting_CPU_Speed
/// Unterliegt aktuell noch Schwankungen um die 15%
#[allow(dead_code)]
/*fn calc_freq() -> usize {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    const SIZE: usize = 50;
    let mut array: [usize; SIZE] = [0; SIZE];
    let mut stsc: usize;
    let mut etsc: usize;
    let lo: usize = 0;
    let hi: usize = 0;
    let mut i = SIZE;
    loop {
        unsafe {
            i -= 1;
            asm!("
                    mov al, 0x34
                    out 0x43, al

                    mov rcx, 30000
                    mov al, cl
                    out 0x40, al
                    mov al, ch
                    out 0x40, al"::::"intel", "volatile");

            stsc = x86_64::instructions::rdtsc() as usize;
            for _i in 0..4000 {
                asm!("  xor eax,edx
                        xor edx,eax"::::"intel", "volatile");
            }
            etsc = x86_64::instructions::rdtsc() as usize;
            //        asm!("
            //                out 0x43, 0x04");

            asm!(""::"{rcx}"(lo),"{rcx}"(hi));
        }
        let ticks: usize = 0x7300 - ((hi * 256) + lo);
        let freq: usize = (((etsc - stsc) * 1193182) / ticks) as usize;
        array[i] = freq;
        if i == 0 {
            break;
        }
    }

    let mut freq: usize = 0;

    for x in array.iter() {
        freq += *x;
    }

    return freq / array.len();
}*/

pub fn calc_freq() -> u64 {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    let mut r0 :u64 = 0;
    let mut f0 :u64 = 0;
    let mut r1 :u64 = 0;
    let mut r2 :u64 = 0;
    let mut t0 :u64 = 0;
    let mut t1 :u64 = 0;
    let mut hi :u64 = 0;
    let mut lo :u64 = 0;
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
                 ":"=r"(reg)::"rax":"intel","volatile");}
    return reg;
}

pub fn get_cpu_freq() -> u64 {
    unsafe {
        if CPU_FREQ == 0 {
            CPU_FREQ = calc_freq();
        }
        CPU_FREQ as u64
    }
}

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