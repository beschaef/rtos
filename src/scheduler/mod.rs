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
use HEAP_ALLOCATOR;

pub struct TaskData {
    cpu_flags: u64,
    stack_pointer: VirtualAddress,
    instruction_pointer: VirtualAddress,
    status: u64,
}
impl TaskData {
    pub const fn new(
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        status: u64,
    ) -> Self {
        TaskData {
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            status,
        }
    }

    pub const fn dummy() -> Self {
        TaskData::new(0,x86_64::VirtualAddress(0),x86_64::VirtualAddress(0),
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
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime1 as usize),

            1,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime2 as usize),

            1,
        ),
    );
}

pub fn uptime1() {
    let color = Color::LightGreen;
    let mut l = 0;
    let mut r = 0;
    loop {
        l = (l +1) %5;
        r = (r +1) %9;
        let color = Color::LightGreen;
//        unsafe {
//            let speicher = HEAP_ALLOCATOR.alloc(&format!("{}",r));
//            vga_buffer::write_at(speicher, 0, 0 + 7, color);
//        }
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
    }
}
pub fn uptime2() {
    let color = Color::LightGreen;
    let mut l = -1;
    let mut r = -1;
    loop {
        l = (l +1) %5;
//        r = (r +1) %9;
//        let color = Color::LightGreen;
//        match r {
//            0 => vga_buffer::write_at("1", 2, 0 + 7, color),
//            1 => vga_buffer::write_at("2", 2, 0 + 7, color),
//            2 => vga_buffer::write_at("3", 2, 0 + 7, color),
//            3 => vga_buffer::write_at("4", 2, 0 + 7, color),
//            4 => vga_buffer::write_at("5", 2, 0 + 7, color),
//            5 => vga_buffer::write_at("6", 2, 0 + 7, color),
//            6 => vga_buffer::write_at("7", 2, 0 + 7, color),
//            7 => vga_buffer::write_at("8", 2, 0 + 7, color),
//            8 => vga_buffer::write_at("9", 2, 0 + 7, color),
//            9 => vga_buffer::write_at("0", 2, 0 + 7, color),
//            _ => vga_buffer::write_at("0", 2, 0 + 7, color),
//        }
    }
}

pub fn schedule(f: &mut ExceptionStackFrame) {
    let cpuflags = f.cpu_flags;
    let stackpointer = f.stack_pointer;
    let instructionpointer = f.instruction_pointer;
    let running = TASKS.lock().pop().unwrap();
    unsafe {
        if RUNNING_TASK.lock().status != 0 {
            let old = TaskData::new(
                cpuflags,
                stackpointer,
                instructionpointer,
                running.status,
            );
            TASKS.lock().insert(0, old);
        }
        f.stack_pointer = running.stack_pointer;
        f.instruction_pointer = running.instruction_pointer;

        RUNNING_TASK.lock().status = running.status;
        RUNNING_TASK.lock().stack_pointer = running.stack_pointer;
        RUNNING_TASK.lock().instruction_pointer = running.instruction_pointer;
    }
}
