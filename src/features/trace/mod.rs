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
        for x in format!("{:<20} - {:20?} - {:?}\n",fn_name,ts, info_text).bytes() {
            unsafe{
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }
}

pub fn trace_begin(fn_name: &str) {
    TRACE.lock().write(fn_name, "begin");
}

pub fn trace_end(fn_name: &str) {
    TRACE.lock().write(fn_name, "end");
}

pub fn trace_info(fn_name: &str, info_text: &str) {
    TRACE.lock().write(fn_name, info_text);
}


