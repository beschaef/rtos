use alloc::Vec;
use cpuio;
use features::clock::Clock;
use features::keyboard;
use memory::MemoryController;
use pic::ChainedPics;
use spin::{Mutex, Once};
use vga_buffer;
use vga_buffer::Color;
use x86_64;
use x86_64::structures::idt::{ExceptionStackFrame, Idt, PageFaultErrorCode};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;
use x86_64::instructions::rdtsc;
use HEAP_ALLOCATOR;
use features::get_cpu_freq;

static mut PID_COUNTER: usize = 0;

static mut RUNNING_TASK: Mutex<TaskData> = Mutex::new(TaskData {
    pid: 0,
    cpu_flags: 0,
    stack_pointer: x86_64::VirtualAddress(0),
    instruction_pointer: x86_64::VirtualAddress(0),
    status: TaskStatus::RUNNING
});

fn increment_pid() -> usize {
    unsafe {
        PID_COUNTER += 1;
        PID_COUNTER
    }
}

lazy_static! {
    static ref TASKS: Mutex<Vec<TaskData>> = Mutex::new(vec![]);
    static ref IDLE_TASK: Mutex<TaskData> = Mutex::new(TaskData::new(
        0,
        x86_64::VirtualAddress(0),
        x86_64::VirtualAddress(idle_task as usize),
        TaskStatus::IDLE
    ));
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    IDLE,
    READY,
    RUNNING,
    SLEEPING
}

#[derive(Debug, Clone)]
pub struct TaskData {
    pid: usize,
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    instruction_pointer: VirtualAddress,
    status: TaskStatus,
}

///unsafe block is actually safe because we're initializing the tasks before the interrupts are enabled
impl TaskData {
    pub fn new(
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        status: TaskStatus
    ) -> Self {
        TaskData {
            pid: increment_pid(),
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            status,
        }
    }

    pub fn dummy() -> Self {
        TaskData {
            pid: increment_pid(),
            cpu_flags: 0,
            stack_pointer: x86_64::VirtualAddress(0),
            instruction_pointer: x86_64::VirtualAddress(0),
            status: TaskStatus::RUNNING
        }
    }
}

pub fn sched_init(memory_controller: &mut MemoryController) {
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime1 as usize),
            TaskStatus::READY
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime2 as usize),
            TaskStatus::READY
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(idle_task as usize),
            TaskStatus::IDLE
        ),
    );

    early_trace!("initialised scheduler");
}

pub fn uptime1() {
    msleep(1000);
    early_trace!();
    //trace!("started uptime1");

    let color = Color::LightGreen;
    let mut r = 0;
    loop {
        //trace!("loop uptime1");

        r = (r + 1) % 9;
        let color = Color::LightGreen;
        match r {
            0 => vga_buffer::write_at("1", 0, 0 + 7, color),
            1 => vga_buffer::write_at("2", 0, 0 + 7, color),
            2 => vga_buffer::write_at("3", 0, 0 + 7, color),
            3 => vga_buffer::write_at("4", 0, 0 + 7, color),
            4 => vga_buffer::write_at("5", 0, 0 + 7, color),
            5 => vga_buffer::write_at("6", 0, 0 + 7, color),
            6 => vga_buffer::write_at("7", 0, 0 + 7, color),
            7 => vga_buffer::write_at("8", 0, 0 + 7, color),
            8 => vga_buffer::write_at("9", 0, 0 + 7, color),
            9 => vga_buffer::write_at("0", 0, 0 + 7, color),
            _ => vga_buffer::write_at("0", 0, 0 + 7, color),
        }
        early_trace!("CHECK");
        msleep(1000);
    }
}

pub fn uptime2() {
    msleep(1000);
    early_trace!();
    //trace!("started uptime2");
    let color = Color::LightGreen;
    let mut l = -1;
    let mut x = 0;
    loop {
        //trace!("loop uptime2");

        l = (l + 1) % 9;
        let color = Color::LightGreen;
        match l {
            0 => vga_buffer::write_at("1", 2, 0 + 7, color),
            1 => vga_buffer::write_at("2", 2, 0 + 7, color),
            2 => vga_buffer::write_at("3", 2, 0 + 7, color),
            3 => vga_buffer::write_at("4", 2, 0 + 7, color),
            4 => vga_buffer::write_at("5", 2, 0 + 7, color),
            5 => vga_buffer::write_at("6", 2, 0 + 7, color),
            6 => vga_buffer::write_at("7", 2, 0 + 7, color),
            7 => vga_buffer::write_at("8", 2, 0 + 7, color),
            8 => vga_buffer::write_at("9", 2, 0 + 7, color),
            9 => vga_buffer::write_at("0", 2, 0 + 7, color),
            _ => vga_buffer::write_at("0", 2, 0 + 7, color),
        }
        early_trace!("CHECK");
        msleep(1000);
    }
}

fn idle_task(){
    loop{
        unsafe{asm!("pause":::: "intel", "volatile");}
    }
}


pub fn msleep(ms: u64) {
    let one_sec = get_cpu_freq();
//    let time = one_sec * (1000 / ms) + rdtsc();
//    trace!("sleep until {}",time);
//    loop {
//        if time > rdtsc() {
//        }
//        else {
//            break;
//        }
//    }
    let mut time = (one_sec * (ms / 1000)); // (one_sec * ms / 1000) as i64; doesnt work!
    let mut tsc = time + rdtsc();
    //trace!("sleep until {}",time);
    while tsc > rdtsc() {
        unsafe{asm!("INT 20h"::::"intel","volatile");}
    }
    //trace!("after while");
//    return time;

}

pub fn schedule(f: &mut ExceptionStackFrame) {
    early_trace!();
    let cpuflags = f.cpu_flags;
    let stackpointer = f.stack_pointer;
    let instructionpointer = f.instruction_pointer;
    let to_run = TASKS.lock().pop().expect("scheduler schedule failed");
    //trace!("task: {}", to_run.instruction_pointer);
    unsafe {
        if RUNNING_TASK.lock().pid != 0 { // PID = 0 --> main function
            //let old = TaskData::new(cpuflags, stackpointer, instructionpointer, to_run.status);
            let old = RUNNING_TASK.lock().clone();
            TASKS.lock().insert(0, old);
        }
        f.stack_pointer = to_run.stack_pointer;
        f.instruction_pointer = to_run.instruction_pointer;

        RUNNING_TASK.lock().status = to_run.status;
        RUNNING_TASK.lock().stack_pointer = to_run.stack_pointer;
        RUNNING_TASK.lock().instruction_pointer = to_run.instruction_pointer;
    }
}
