use cpuio::UnsafePort;
use spin::Mutex;
use x86_64;
use x86_64::instructions::rdtsc;

struct Trace {
    //port: cpuio::UnsafePort,
}

lazy_static! {
    static ref TRACE: Mutex<Trace> = Mutex::new(Trace {});
    static ref TEMP: u8 = 0;
}

impl Trace {
    pub fn write(&mut self, fn_name: &str, info_text: &str) {
        let ts = rdtsc();
        for x in format!("{:<25} - tsc: {:15?} - {:?}\n", fn_name, ts, info_text).bytes() {
            unsafe {
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }
}

pub fn trace_info(fn_name: &str, info_text: &str) {
    unsafe {
        x86_64::instructions::interrupts::disable();
        trace_info_without_interrupts(fn_name, info_text);
        x86_64::instructions::interrupts::enable();
    }
}

pub fn trace_info_without_interrupts(fn_name: &str, info_text: &str) {
    unsafe {
        let mut lock = TRACE.try_lock();
        if lock.is_some() {
            let mut unwrapped = lock.expect("trace unwrap failed");
            unwrapped.write(fn_name, info_text);
        }
    }
}

macro_rules! trace {
    () => (simple_trace!(""));
    ($fmt:expr) =>           (simple_trace!($fmt));
    ($fmt:expr, $($arg:tt)*) => (simple_trace!($fmt, $($arg)*));

}

macro_rules! simple_trace {
    ($($arg:tt)*) => ($crate::trace::trace_info(function!(),&format!($($arg)*)));
}

/// This Trace isn't disabling the Interrupts while writing.
/// Only use in Interruptroutine's or before enabling Interrupts.
macro_rules! early_trace {
    () => ($crate::trace::trace_info_without_interrupts(function!(),&format!("")));
    ($fmt:expr) => ($crate::trace::trace_info_without_interrupts(function!(),&format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::trace::trace_info_without_interrupts(function!(),&format!($fmt, $($arg)*)));

}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            extern crate core;
            unsafe { core::intrinsics::type_name::<T>() }
        }
        let name = type_name_of(f);
        &name[6..name.len() - 4]
    }};
}
