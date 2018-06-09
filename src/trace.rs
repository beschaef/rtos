use cpuio::UnsafePort;
use x86_64::instructions::rdtsc;
use spin::Mutex;

struct Trace {
    //port: cpuio::UnsafePort,
}

lazy_static! {
    static ref TRACE: Mutex<Trace> = Mutex::new(Trace{});
    static ref TEMP: u8 = 0;
}

impl Trace {

    pub fn write(&mut self, fn_name: &str, info_text: &str) {
        let ts = rdtsc();
        for x in format!("{:<25} - tsc: {:15?} - {:?}\n",fn_name,ts, info_text).bytes() {
            unsafe{
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }
}

pub fn trace_info(fn_name: &str, info_text: &str) {
    TRACE.lock().write(fn_name, info_text);
}


macro_rules! trace {
    () => (simple_trace!(""));
    ($fmt:expr) =>           (simple_trace!($fmt));
    ($fmt:expr, $($arg:tt)*) => (simple_trace!($fmt, $($arg)*));

}

macro_rules! simple_trace {
    ($($arg:tt)*) => ($crate::trace::trace_info(function!(),&format!($($arg)*)));
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
    }}
}


