pub mod clock;
pub mod keyboard;

use x86_64;

static mut CPU_FREQ: usize = 0;

/// Wird genutzt um die cpu_frequenz zu berechnen.
/// Code ist angelehnt an https://wiki.osdev.org/Detecting_CPU_Speed
/// Unterliegt aktuell noch Schwankungen um die 15%
///
pub fn calc_freq() -> usize {
    unsafe {
        x86_64::instructions::interrupts::disable();
    }
    const SIZE: usize = 50;
    let mut array: [usize; SIZE] = [0; SIZE];
    let mut stsc: usize = 0;
    let mut etsc: usize = 0;
    let mut lo: usize = 0;
    let mut hi: usize = 0;
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
            for i in 0..4000 {
                asm!("  xor eax,edx
                        xor edx,eax"::::"intel", "volatile");
            }
            etsc = x86_64::instructions::rdtsc() as usize;
            //        asm!("
            //                out 0x43, 0x04");

            asm!(""::"{rcx}"(lo),"{rcx}"(hi));
        }
        let ticks: usize = (0x7300 - ((hi * 256) + lo));
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
}

pub fn get_cpu_freq() -> u64 {
    unsafe {
        if CPU_FREQ == 0 {
            calc_freq();
            CPU_FREQ = calc_freq();
        }
        CPU_FREQ as u64
    }
}
