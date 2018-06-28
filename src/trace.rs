//! This module is used to trace informations. All data is written to port `0x03f8`.
//! It's possible to trace five different level of tracing. `Debug`, `Info`, `Warn`, `Error`,
//! `Fatal`, and `None`.
//! If the Trace level is set to `None`, nothing is traced.
//! For easier usage there are different macros for each trace level.
//! There is also a macro to change the trace level while the system is running.
use cpuio::UnsafePort;
use spin::Mutex;
use x86_64;
use x86_64::instructions::rdtsc;

/// the serial port to write is fix, so there is no need to store any data in the struct.
struct Trace {
    //port: cpuio::UnsafePort,
}

/// Same Level as the ROS (Robot Operating System) Log level.
/// Debug to Fatal are used to Trace different Types.
/// If Trace Level is set to None no Traces are written.
/// The smaller the Tracelevel is the more Infos are traced.
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

/// Global Variable to store the actual Trace level.
pub static mut TRACE_LEVEL: TraceLevel = TraceLevel::Info;

lazy_static! {
    /// Global Trace struct, saved with a Mutex, to use it in all Files and to
    /// pretend race conditions or similar multi access problems.
    static ref TRACE: Mutex<Trace> = Mutex::new(Trace {});
}

impl Trace {
    /// This function builds a string with the arguments and writes then all bytes on Port `0x03f8`.
    ///
    /// # Output example
    ///
    /// Info: module:function_name - tsc: 1234567890 - Some additional info text
    ///
    /// # Arguments
    /// * `level` - writes trace level ('Info' in example).
    /// * `fn_name` - writes function name ('module:function_name' in example)
    /// * `info_text` - writes additional info ('Some additional info text' in example).
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

/// writes a trace with disabled interrupts.
///
/// # Arguments
/// * `level` - writes trace level ('Info' in example).
/// * `fn_name` - writes function name ('module:function_name' in example)
/// * `info_text` - writes additional info ('Some additional info text' in example).
pub fn trace_info(level: &str, fn_name: &str, info_text: &str) {
    unsafe {
        x86_64::instructions::interrupts::disable();
        trace_info_without_interrupts(level, fn_name, info_text);
        x86_64::instructions::interrupts::enable();
    }
}

/// writes a trace with given arguments.
///
/// # Arguments
/// * `level` - writes trace level ('Info' in example).
/// * `fn_name` - writes function name ('module:function_name' in example)
/// * `info_text` - writes additional info ('Some additional info text' in example).
pub fn trace_info_without_interrupts(level: &str, fn_name: &str, info_text: &str) {
    let lock = TRACE.try_lock();
    if lock.is_some() {
        let mut unwrapped = lock.expect("trace unwrap failed");
        unwrapped.write(level, fn_name, info_text);
    }
}

/// traces a given message when trace level is set to `Debug`
///
/// # Output example
///
/// Debug: module:function_name - tsc: 1234567890 - Some debug info
///
/// # Examples
/// trace_fatal!();
///
/// trace_fatal!("Some debug info");
///
/// trace_fatal!("Some {} info", "debug");
///
#[macro_export]
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

/// traces a given message when trace level is set to `Info` or `Debug`
///
/// # Output example
///
/// Info: module:function_name - tsc: 1234567890 - Awesome stuff is happend
///
/// # Examples
/// trace_fatal!();
///
/// trace_fatal!("Awesome stuff is happend");
///
/// trace_fatal!("Awesome stuff is {}", "happend");
///
#[macro_export]
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

/// traces a given message when trace level is set to `Warn`, `Info` or `Debug`
///
/// # Output example
///
/// Warn: module:function_name - tsc: 1234567890 - Something happend
///
/// # Examples
/// trace_fatal!();
///
/// trace_fatal!("Something happend");
///
/// trace_fatal!("Something {}", "happend");
///
#[macro_export]
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

/// traces a given message when trace level is set to `Error`, `Warn`, `Info` or `Debug`
///
/// # Output example
///
/// Error: module:function_name - tsc: 1234567890 - Something bad happend
///
/// # Examples
/// trace_fatal!();
///
/// trace_fatal!("Something bad happend");
///
/// trace_fatal!("Something {} happend", "bad");
///
#[macro_export]
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

/// traces a given message when trace level is set to `Fatal`, `Error`, `Warn`, `Info` or `Debug`
///
/// # Output example
///
/// Fatal: module:function_name - tsc: 1234567890 - Something really bad happend
///
/// # Examples
/// trace_fatal!();
///
/// trace_fatal!("Something really bad happend");
///
/// trace_fatal!("Something really {} happend", "bad");
///
#[macro_export]
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

/// traces a given message by using the trace::trace_info() function
///
/// # Examples
/// simple_trace!("Debug", "Some info Text");
#[macro_export]
macro_rules! simple_trace {
    ($a:expr, $($arg:tt)*) => ($crate::trace::trace_info(&format!($a), function!(),&format!($($arg)*)));
}

/// traces a given message by using the trace::trace_info_without_interrupts() function
///
/// # Examples
/// early_trace!();
///
/// early_trace!("Some info Text");
///
/// early_trace!("Some info {}", "Text");
///
#[allow(dead_code)]
#[macro_export]
macro_rules! early_trace {
    () => ($crate::trace::trace_info_without_interrupts("",function!(),&format!("")));
    ($fmt:expr) => ($crate::trace::trace_info_without_interrupts("",function!(),&format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::trace::trace_info_without_interrupts("",function!(),&format!($fmt, $($arg)*)));

}

/// changes the trace level
///
/// # Examples
/// set_trace_level!(TraceLevel::Warn);
///
/// set_trace_level!(TraceLevel::None);
#[macro_export]
macro_rules! set_trace_level {
    ($e:expr) => {
        use core::any::Any;
        if let Some(f) = (&$e as &Any).downcast_ref::<TraceLevel>() {
            unsafe{TRACE_LEVEL = *f;}
        }
    };
}

/// returns the function and module from where it's called.
///
/// # Examples
/// mod foo {
///
///     def bar() {
///
///         function!() # will return foo:bar
///
///     }
///
/// }
///
#[macro_export]
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
