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

    pub fn begin(&mut self, fn_name: &str) {
        let ts = rdtsc();
        for x in format!("{:20?} - {:20?} - begin\n",fn_name,ts).bytes() {
            unsafe{
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }

    pub fn end(&mut self, fn_name: &str) {
        let ts = rdtsc();
        for x in format!("{:20?} - {:20?} - end\n",fn_name,ts).bytes() {
            unsafe{
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }

    pub fn info(&mut self, fn_name: &str, info_text: &str) {
        let ts = rdtsc();
        for x in format!("{:20?} - {:20?} - {:?}\n",fn_name,ts, info_text).bytes() {
            unsafe{
                UnsafePort::new(0x03f8).write(x);
            }
        }
    }
}

pub fn trace_begin(fn_name: &str) {
    TRACE.lock().begin(fn_name);
}

pub fn trace_end(fn_name: &str) {
    TRACE.lock().end(fn_name);
}

pub fn trace_info(fn_name: &str, info_text: &str) {
    TRACE.lock().info(fn_name, info_text);
}


