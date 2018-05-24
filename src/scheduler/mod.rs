use cpuio;
use features::keyboard;
use memory::MemoryController;
use pic::ChainedPics;
use spin::{Mutex, Once};
use x86_64::structures::idt::{ExceptionStackFrame, Idt, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;
use x86_64;
use alloc::Vec;
use features::clock::Clock;

static mut COUNTER: usize = 0;
static mut COUNTER_SMALL: usize = 0;
static mut TIME: usize = 0;

struct TaskData {
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    instruction_pointer: VirtualAddress,
    code_segment: u64,
    stack_segment: u64,
}
impl TaskData {
    pub fn new(cpu_flags: u64,
               stack_pointer: VirtualAddress,
               instruction_pointer: VirtualAddress,
               code_segment: u64,
               stack_segment: u64,) -> Self {
        TaskData{cpu_flags,
            stack_pointer,
            instruction_pointer,
            code_segment,
            stack_segment,}
    }
}

lazy_static! {
    static ref TASKS: Vec<((), TaskData)> = {
        let mut tasks = vec!();
        let mut clock1 = Clock::new(0,0);
        let mut clock2 = Clock::new(0,71);
        tasks.push((clock1.uptime(), TaskData::new(0,x86_64::VirtualAddress(0),x86_64::VirtualAddress(0),0,0)));
        tasks.push((clock2.uptime(), TaskData::new(0,x86_64::VirtualAddress(0),x86_64::VirtualAddress(0),0,0)));
        tasks
    };
}

pub fn schedule(f:  &mut ExceptionStackFrame) {

    let x = f.cpu_flags;
    let y = f.stack_pointer;
    let x = f.instruction_pointer;
    let x = f.code_segment;
    let x = f.stack_segment;
    let running = TASKS.pop();
    unsafe {
        COUNTER += 1;
        //let one_second = if COUNTER_SMALL % 3 == 0 {
        //   118 } else { 119};
        let one_second = 119;
        if COUNTER % one_second == 0 {
            COUNTER_SMALL += 1;
            TIME += 1;
            println!("{}",TIME);
            //println!("{}",x86_64::instructions::rdtsc());
        }
    }
}