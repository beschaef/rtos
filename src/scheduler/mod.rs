use alloc::Vec;
use memory::MemoryController;
use spin::{Mutex};
use x86_64;
use x86_64::instructions::rdtsc;
use x86_64::structures::idt::{ExceptionStackFrame};
use tasks::*;

pub static mut RUNNING_TASK: Mutex<TaskData> = Mutex::new(TaskData {
    pid: 0,
    cpu_flags: 0,
    stack_pointer: x86_64::VirtualAddress(0),
    instruction_pointer: x86_64::VirtualAddress(0),
    status: TaskStatus::RUNNING,
    sleep_ticks: 0,
});

lazy_static! {
    static ref TASKS: Mutex<Vec<TaskData>> = Mutex::new(vec![]);
}

pub fn sched_init(memory_controller: &mut MemoryController) {
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime1 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime2 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime3 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime4 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime_temp as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(tetris as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(idle_task as usize),
            TaskStatus::IDLE,
        ),
    );

    early_trace!("initialised scheduler");
}

pub fn schedule(f: &mut ExceptionStackFrame) {
    //early_trace!();
    let cpuflags = f.cpu_flags;
    let stackpointer = f.stack_pointer;
    let instructionpointer = f.instruction_pointer;
    // check if a task is ready to run
    let tsc = rdtsc();
    let to_run = if TASKS.lock().last().expect("last").status == TaskStatus::READY {
        early_trace!("popped ready task");
        let x = TASKS.lock().pop().expect("popped");
        x
    } else if TASKS.lock().last().expect("tadaa").sleep_ticks < tsc as usize {
        let x = TASKS.lock().pop().expect("popped");
        early_trace!("popped after sleep task {}", x.sleep_ticks);
        x
    } else if unsafe{RUNNING_TASK.lock().status == TaskStatus::IDLE} {
        //early_trace!("do nothing");
        return;
    } else {
        early_trace!("popped idle");
        let x = TASKS.lock().remove(0);
        x
    };
    //let to_run = TASKS.lock().pop().expect("scheduler schedule failed");
    //trace!("task: {}", to_run.instruction_pointer);

    unsafe {
        let not_finished = RUNNING_TASK.lock().status != TaskStatus::FINISHED;
        if RUNNING_TASK.lock().pid != 0 && not_finished {
            let pid_c = RUNNING_TASK.lock().pid;
            let sleep_ticks_c = RUNNING_TASK.lock().sleep_ticks;
            // PID = 0 --> main function
            //let old = TaskData::new(cpuflags, stackpointer, instructionpointer, to_run.status);
            let new_status = if RUNNING_TASK.lock().status == TaskStatus::IDLE {
                TaskStatus::IDLE
            } else {
                TaskStatus::RUNNING
            };
            let old = TaskData::copy(
                pid_c,
                cpuflags,
                stackpointer,
                instructionpointer,
                new_status,
                sleep_ticks_c,
            );
            let mut position = 0;
            if old.status == TaskStatus::IDLE{
                TASKS.lock().insert(position, old);
            } else {
                for task in TASKS.lock().iter() {
                    if task.sleep_ticks <= sleep_ticks_c && task.status != TaskStatus::IDLE{
                        break;
                    }
                    position += 1;
                }
                TASKS.lock().insert(position, old);
            }
        }
        f.stack_pointer = to_run.stack_pointer;
        f.instruction_pointer = to_run.instruction_pointer;

        RUNNING_TASK.lock().status = to_run.status;
        RUNNING_TASK.lock().stack_pointer = to_run.stack_pointer;
        RUNNING_TASK.lock().instruction_pointer = to_run.instruction_pointer;
        RUNNING_TASK.lock().pid = to_run.pid;
        RUNNING_TASK.lock().sleep_ticks = to_run.sleep_ticks;
    }
}
