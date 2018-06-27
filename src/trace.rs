use cpuio::UnsafePort;
use spin::Mutex;
use x86_64;
use x86_64::instructions::rdtsc;
use core::any::Any;

struct Trace {
    //port: cpuio::UnsafePort,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub enum TraceLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Fatal = 4,
    None = 5,
}

pub static mut TRACE_LEVEL: TraceLevel = TraceLevel::Info;

lazy_static! {
    static ref TRACE: Mutex<Trace> = Mutex::new(Trace {});
}

impl Trace {
    pub fn write(&mut self, level: &str, fn_name: &str, info_text: &str) {
        let ts = rdtsc();
        for x in format!(
            "{:<5}: {:<25} - tsc: {:15?} - {:?}\n",
            level, fn_name, ts, info_text
        ).bytes()
        {
            unsafe {
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }
}

pub fn trace_info(level: &str, fn_name: &str, info_text: &str) {
    unsafe {
        x86_64::instructions::interrupts::disable();
        trace_info_without_interrupts(level, fn_name, info_text);
        x86_64::instructions::interrupts::enable();
    }
}

pub fn trace_info_without_interrupts(level: &str, fn_name: &str, info_text: &str) {
    let lock = TRACE.try_lock();
    if lock.is_some() {
        let mut unwrapped = lock.expect("trace unwrap failed");
        unwrapped.write(level, fn_name, info_text);
    }
}

macro_rules! trace {
    () => (simple_trace!("",""));
    ($fmt:expr) =>           (simple_trace!("",$fmt));
    ($fmt:expr, $($arg:tt)*) => (simple_trace!("",$fmt, $($arg)*));
}

macro_rules! trace_debug {
    () => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Debug {
            (simple_trace!("Debug",""))
        }
    };
    ($fmt:expr) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Debug {
            (simple_trace!("Debug",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Debug {
            (simple_trace!("Debug",$fmt, $($arg)*))
        }
    };
}

macro_rules! trace_info {
    () => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Info {
            (simple_trace!("Info",""))
        }
    };
    ($fmt:expr) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Info {
            (simple_trace!("Info",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Info {
            (simple_trace!("Info",$fmt, $($arg)*))
        }
    };
}

macro_rules! trace_warn {
    () => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Warn {
            (simple_trace!("Warn",""))
        }
    };
    ($fmt:expr) => {
        if unsafe{TRACE_LEVEL} <= TraceLevel::Warn {
            (simple_trace!("Warn",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Warn {
            (simple_trace!("Warn",$fmt, $($arg)*))
        }
    };
}

macro_rules! trace_error {
    () => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Error {
            (simple_trace!("Error",""))
        }
    };
    ($fmt:expr) => {
        if unsafe{TRACE_LEVEL} <= TraceLevel::Error {
            (simple_trace!("Error",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Error {
            (simple_trace!("Error",$fmt, $($arg)*))
        }
    };
}

macro_rules! trace_fatal {
    () => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Fatal {
            (simple_trace!("Fatal",""))
        }
    };
    ($fmt:expr) => {
        if unsafe{TRACE_LEVEL} <= TraceLevel::Fatal {
            (simple_trace!("Fatal",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Fatal {
            (simple_trace!("Fatal",$fmt, $($arg)*))
        }
    };
}

macro_rules! simple_trace {
    ($a:expr, $($arg:tt)*) => ($crate::trace::trace_info(&format!($a), function!(),&format!($($arg)*)));
}

/// This Trace isn't disabling the Interrupts while writing.
/// Only use in Interruptroutine's or before enabling Interrupts.
macro_rules! early_trace {
    () => ($crate::trace::trace_info_without_interrupts("",function!(),&format!("")));
    ($fmt:expr) => ($crate::trace::trace_info_without_interrupts("",function!(),&format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::trace::trace_info_without_interrupts("",function!(),&format!($fmt, $($arg)*)));

}

macro_rules! set_trace_level {
    ($e:expr) => {
        use core::any::Any;
        if let Some(f) = (&$e as &Any).downcast_ref::<TraceLevel>() {
            unsafe{TRACE_LEVEL = *f;}
        }
    };
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
