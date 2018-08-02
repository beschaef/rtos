//! This module is used to trace information. All data is written to port `0x03f8`.
//! It's possible to use five different level of tracing: `Debug`, `Info`, `Warn`, `Error`,
//! `Fatal`, and `None`.
//! If the Trace level is set to `None`, nothing is traced.
//! For easier usage there are different macros for each trace level.
//! There is also a macro to change the trace level while the system is running.
use cpuio::UnsafePort;
use spin::Mutex;
use x86_64;
use x86_64::instructions::rdtsc;

/// The serial port to write is fix, so there is no need to store any data in the struct.
struct Trace {
    //port: cpuio::UnsafePort,
}

/// The same level as the ROS (Robot Operating System) log levels.
/// Debug to *Fatal* are used to trace different types.
/// If the trace level is set to *None*, no traces are written.
/// The *smaller* the trace level, the more information is traced.
#[allow(dead_code)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub enum TraceLevel {
    /// For debugging information.
    Debug = 0,
    /// For additional information.
    Info = 1,
    /// For warnings which are needed to trace, but won't kill the system.
    Warn = 2,
    /// For errors which will kill the system.
    Error = 3,
    /// For fatal errors or important system information.
    Fatal = 4,
    /// Used when nothing should be traced.
    None = 5,
}

/// Global variable to store the trace level.
pub static mut TRACE_LEVEL: TraceLevel = TraceLevel::Info;

lazy_static! {
    /// Global Trace struct, guarded by a Mutex, to use it in all files and to
    /// prevent race conditions or similar multi-access problems.
    static ref TRACE: Mutex<Trace> = Mutex::new(Trace {});
}

impl Trace {
    /// This function builds a string with the arguments and then writes all bytes on Port `0x03f8`.
    ///
    /// # Output example
    ///
    /// Info: module:function_name - tsc: 1234567890 - Some additional info text
    ///
    /// # Arguments
    /// * `level` - (&str) Trace level ('Info' in the example).
    /// * `fn_name` - (&str) Function name ('module:function_name' in the  example)
    /// * `info_text` - (&str) Additional info ('Some additional info text' in the example).
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

/// Writes a trace with disabled interrupts.
///
/// # Arguments
/// * `level` - (&str) Trace level ('Info' in the example).
/// * `fn_name` - (&str) Function name ('module:function_name' in the  example)
/// * `info_text` - (&str) Additional info ('Some additional info text' in the example).
pub fn trace_info(level: &str, fn_name: &str, info_text: &str) {
    unsafe {
        x86_64::instructions::interrupts::disable();
        trace_info_without_interrupts(level, fn_name, info_text);
        x86_64::instructions::interrupts::enable();
    }
}

/// Writes a trace with given arguments.
///
/// # Arguments
/// * `level` - (&str) Trace level ('Info' in the example).
/// * `fn_name` - (&str) Function name ('module:function_name' in the  example)
/// * `info_text` - (&str) Additional info ('Some additional info text' in the example).
pub fn trace_info_without_interrupts(level: &str, fn_name: &str, info_text: &str) {
    let lock = TRACE.try_lock();
    if lock.is_some() {
        let mut unwrapped = lock.expect("trace unwrap failed");
        unwrapped.write(level, fn_name, info_text);
    }
}

/// Traces a given message when trace level is set to `Debug`.
///
/// # Output example
///
/// Debug: module:function_name - tsc: 1234567890 - Some debug info
///
/// # Examples
/// trace_debug!();
///
/// trace_debug!("Some debug info");
///
/// trace_debug!("Some {} info", "debug");
///
#[macro_export]
macro_rules! trace_debug {
    () => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Debug {
            (simple_trace!("Debug",""))
        }
    };
    ($fmt:expr) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Debug {
            (simple_trace!("Debug",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Debug {
            (simple_trace!("Debug",$fmt, $($arg)*))
        }
    };
}

/// Traces a given message when trace level is set to `Info` or `Debug`.
///
/// # Output example
///
/// Info: module:function_name - tsc: 1234567890 - Awesome stuff has happened
///
/// # Examples
/// trace_info!();
///
/// trace_info!("Awesome stuff has happened");
///
/// trace_info!("Awesome stuff has {}", "happened");
///
#[macro_export]
macro_rules! trace_info {
    () => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Info {
            (simple_trace!("Info",""))
        }
    };
    ($fmt:expr) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Info {
            (simple_trace!("Info",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Info {
            (simple_trace!("Info",$fmt, $($arg)*))
        }
    };
}

/// Traces a given message when trace level is set to `Warn`, `Info` or `Debug`.
///
/// # Output example
///
/// Warn: module:function_name - tsc: 1234567890 - Something happened
///
/// # Examples
/// trace_warn!();
///
/// trace_warn!("Something happened");
///
/// trace_warn!("Something {}", "happened");
///
#[macro_export]
macro_rules! trace_warn {
    () => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Warn {
            (simple_trace!("Warn",""))
        }
    };
    ($fmt:expr) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Warn {
            (simple_trace!("Warn",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Warn {
            (simple_trace!("Warn",$fmt, $($arg)*))
        }
    };
}

/// Traces a given message when trace level is set to `Error`, `Warn`, `Info` or `Debug`.
///
/// # Output example
///
/// Error: module:function_name - tsc: 1234567890 - Something bad happened
///
/// # Examples
/// trace_error!();
///
/// trace_error!("Something bad happend");
///
/// trace_error!("Something {} happened", "bad");
///
#[macro_export]
macro_rules! trace_error {
    () => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Error {
            (simple_trace!("Error",""))
        }
    };
    ($fmt:expr) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Error {
            (simple_trace!("Error",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Error {
            (simple_trace!("Error",$fmt, $($arg)*))
        }
    };
}

/// Traces a given message when trace level is set to `Fatal`, `Error`, `Warn`, `Info` or `Debug`.
///
/// # Output example
///
/// Fatal: module:function_name - tsc: 1234567890 - Something really bad happened
///
/// # Examples
/// trace_fatal!();
///
/// trace_fatal!("Something really bad happened");
///
/// trace_fatal!("Something really {} happened", "bad");
///
#[macro_export]
macro_rules! trace_fatal {
    () => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Fatal {
            (simple_trace!("Fatal",""))
        }
    };
    ($fmt:expr) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Fatal {
            (simple_trace!("Fatal",$fmt))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[allow(unused_imports)]
        use trace::*;
        if unsafe{TRACE_LEVEL} <= TraceLevel::Fatal {
            (simple_trace!("Fatal",$fmt, $($arg)*))
        }
    };
}

/// Traces a given message by using the trace::trace_info() function.
///
/// # Examples
/// simple_trace!("Debug", "Some info Text");
#[macro_export]
macro_rules! simple_trace {
    ($a:expr, $($arg:tt)*) => ($crate::trace::trace_info(&format!($a), function!(),&format!($($arg)*)));
}

/// Traces a given message by using the function `trace::trace_info_without_interrupts()`.
///
/// # Examples
/// early_trace!();
///
/// early_trace!("Some info text");
///
/// early_trace!("Some info {}", "text");
///
#[allow(dead_code)]
#[macro_export]
macro_rules! early_trace {
    () => ($crate::trace::trace_info_without_interrupts("",function!(),&format!("")));
    ($fmt:expr) => ($crate::trace::trace_info_without_interrupts("",function!(),&format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => ($crate::trace::trace_info_without_interrupts("",function!(),&format!($fmt, $($arg)*)));

}

/// Changes the trace level.
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
            unsafe {
                TRACE_LEVEL = *f;
            }
        }
    };
}

/// Returns the function name and the corresponding module name.
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
