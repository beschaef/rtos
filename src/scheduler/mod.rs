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

static mut COUNTER: u64 = 0;
static mut COUNTER_SMALL: usize = 0;
static mut TIME: usize = 0;

pub struct TaskData {
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    instruction_pointer: VirtualAddress,
    code_segment: u64,
    stack_segment: u64,
    status: u64,
}
impl TaskData {
    pub const fn new(
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        code_segment: u64,
        stack_segment: u64,
        status: u64,
    ) -> Self {
        TaskData {
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            code_segment,
            stack_segment,
            status,
        }
    }

    pub const fn dummy() -> Self {
        TaskData::new(0,x86_64::VirtualAddress(0),x86_64::VirtualAddress(0),        0,
        0,
        0,
        )
    }
}

pub static mut RUNNING_TASK: Mutex<TaskData> =  Mutex::new(TaskData::dummy());
lazy_static! {
    static ref TASKS: Mutex<Vec<TaskData>> = Mutex::new(vec![]);
    static ref NO_MAIN: u8 = 0;
}

pub fn sched_init(memory_controller: &mut MemoryController) {
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime1 as usize),
            0,
            0,
            1,
        ),
    );
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime2 as usize),
            0,
            0,
            1,
        ),
    );
}

pub fn uptime1() {
    let color = Color::LightGreen;
    loop {
        unsafe {
            x86_64::instructions::interrupts::disable();
        }
        match vga_buffer::read_at(0 as usize, (0 + 7) as usize) {
            48 => vga_buffer::write_at("1", 0, 0 + 7, color),
            49 => vga_buffer::write_at("2", 0, 0 + 7, color),
            50 => vga_buffer::write_at("3", 0, 0 + 7, color),
            51 => vga_buffer::write_at("4", 0, 0 + 7, color),
            52 => vga_buffer::write_at("5", 0, 0 + 7, color),
            53 => vga_buffer::write_at("6", 0, 0 + 7, color),
            54 => vga_buffer::write_at("7", 0, 0 + 7, color),
            55 => vga_buffer::write_at("8", 0, 0 + 7, color),
            56 => vga_buffer::write_at("9", 0, 0 + 7, color),
            57 => {
                vga_buffer::write_at("0", 0, 0 + 7, color);
                match vga_buffer::read_at(0 as usize, (0 + 6) as usize) {
                    48 => vga_buffer::write_at("1", 0, 0 + 6, color),
                    49 => vga_buffer::write_at("2", 0, 0 + 6, color),
                    50 => vga_buffer::write_at("3", 0, 0 + 6, color),
                    51 => vga_buffer::write_at("4", 0, 0 + 6, color),
                    52 => vga_buffer::write_at("5", 0, 0 + 6, color),
                    53 => {
                        vga_buffer::write_at("0", 0, 0 + 6, color);
                    }
                    _ => vga_buffer::write_at("0", 0, 0 + 6, color),
                }
            }
            _ => vga_buffer::write_at("0", 0, 0 + 7, color),
        }
        unsafe {
            x86_64::instructions::interrupts::enable();
        }
    }
    /*loop{
        unsafe{
            if COUNTER % 50 == 0{
                vga_buffer::write_at("1", 10, 0 + 7, color);
                COUNTER_SMALL = (COUNTER_SMALL + 1) %5;
            }
            if COUNTER % 100 == 0{
                vga_buffer::write_at("2", 10, 0 + 7, color);
                COUNTER_SMALL = (COUNTER_SMALL + 1) %5;
            }
        }
    }*/
}
pub fn uptime2() {
    let color = Color::LightGreen;
    loop {
        unsafe {
            x86_64::instructions::interrupts::disable();
        }
        match vga_buffer::read_at(2 as usize, (0 + 7) as usize) {
            48 => vga_buffer::write_at("1", 2, 0 + 7, color),
            49 => vga_buffer::write_at("2", 2, 0 + 7, color),
            50 => vga_buffer::write_at("3", 2, 0 + 7, color),
            51 => vga_buffer::write_at("4", 2, 0 + 7, color),
            52 => vga_buffer::write_at("5", 2, 0 + 7, color),
            53 => vga_buffer::write_at("6", 2, 0 + 7, color),
            54 => vga_buffer::write_at("7", 2, 0 + 7, color),
            55 => vga_buffer::write_at("8", 2, 0 + 7, color),
            56 => vga_buffer::write_at("9", 2, 0 + 7, color),
            57 => {
                vga_buffer::write_at("0", 2, 0 + 7, color);
                match vga_buffer::read_at(2 as usize, (0 + 6) as usize) {
                    48 => vga_buffer::write_at("1", 2, 0 + 6, color),
                    49 => vga_buffer::write_at("2", 2, 0 + 6, color),
                    50 => vga_buffer::write_at("3", 2, 0 + 6, color),
                    51 => vga_buffer::write_at("4", 2, 0 + 6, color),
                    52 => vga_buffer::write_at("5", 2, 0 + 6, color),
                    53 => {
                        vga_buffer::write_at("0", 2, 0 + 6, color);
                    }
                    _ => vga_buffer::write_at("0", 2, 0 + 6, color),
                }
            }
            _ => vga_buffer::write_at("0", 2, 0 + 7, color),
        }
        unsafe {
            x86_64::instructions::interrupts::enable();
        }
    }
    /*loop{unsafe{COUNTER = (COUNTER + 1) % 1000}}*/
}

pub fn schedule(f: &mut ExceptionStackFrame) {
    let cpuflags = f.cpu_flags;
    let stackpointer = f.stack_pointer;
    let instructionpointer = f.instruction_pointer;
    let codesegment = f.code_segment;
    let stacksegment = f.stack_segment;
    let running = TASKS.lock().pop().unwrap();
    unsafe {
        if RUNNING_TASK.lock().status != 0 {
            let old = TaskData::new(
                cpuflags,
                stackpointer,
                instructionpointer,
                codesegment,
                stacksegment,
                running.status,
            );
            TASKS.lock().insert(0, old);
        }
        f.stack_pointer = running.stack_pointer;
        f.instruction_pointer = running.instruction_pointer;
        RUNNING_TASK.lock().status = running.status;
        RUNNING_TASK.lock().stack_pointer = running.stack_pointer;
        RUNNING_TASK.lock().instruction_pointer = running.instruction_pointer;

        COUNTER += 1;
        //let one_second = if COUNTER_SMALL % 3 == 0 {
        //   118 } else { 119};
        let one_second = 119;
        if COUNTER % one_second == 0 {
            COUNTER_SMALL += 1;
            TIME += 1;
            //println!("{}", TIME);
            //println!("{}",x86_64::instructions::rdtsc());
        }
    }
}
