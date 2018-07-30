//! # Module Scheduler
//!
//! This module stores all tasks and handle (schedule) all tasks.
//! currently this module only supports EDF scheduling.
//!
use alloc::Vec;
use memory::MemoryController;
use spin::Mutex;
use tasks::*;
use x86_64;
use x86_64::instructions::rdtsc;
use x86_64::structures::idt::ExceptionStackFrame;

/// global variable with informations about the current task.
/// used, inter alia, to remember the sleep ticks for the scheduler.
pub static mut RUNNING_TASK: Mutex<TaskData> = Mutex::new(TaskData {
    name: 'x',
    pid: 0,
    cpu_flags: 0,
    stack_pointer: x86_64::VirtualAddress(0),
    instruction_pointer: x86_64::VirtualAddress(0),
    status: TaskStatus::READY,
    sleep_ticks: 0,
    time_sleep: 1,
    time_active: 1,
    last_time_stamp: 1,
});

lazy_static! {
    /// global vector which stores all current tasks.
    /// all tasks are sorted from: IDLE -> max sleep -> min sleep -> READY
    pub static ref TASKS: Mutex<Vec<TaskData>> = Mutex::new(vec![]);
}

/// used to initialize tasks.
/// for every task (exluding tetris) the function allocates 2 pages (8192B), and then insert a
/// new TaskData into the `TASKS` vector. Therefore the `stack_pointer` (top address of the
/// allocated memory) and the `instruction_pointer` (the function) as usize are stored. Also all
/// Tasks are inserted with TaskStatus `READY` (excluding the idle task, which has all time the
/// TaskStatus `IDLE`)
///
/// # Arguments
/// * `memory_controller` - needed (and used) to allocate memory.
///
pub fn sched_init(memory_controller: &mut MemoryController) {
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            '1',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime1 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(5).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            '2',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime2 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            '3',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime3 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            '4',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(uptime4 as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            'k',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(task_keyboard as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            's',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(shell as usize),
            TaskStatus::READY,
        ),
    );
    let memory = memory_controller.alloc_stack(2).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            'i',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(idle_task as usize),
            TaskStatus::IDLE,
        ),
    );
    let memory = memory_controller.alloc_stack(3).expect("Ooopsie");
    TASKS.lock().insert(
        0,
        TaskData::new(
            'h',
            0,
            x86_64::VirtualAddress(memory.top()),
            x86_64::VirtualAddress(htop as usize),
            TaskStatus::READY,
        ),
    );
    trace_info!("initialised scheduler");
}

/// used to schedule all tasks.
/// therefore the function saves the `cpu_flags`, `stack_pointer` and `instruction_pointer` given by
/// the timer interrupt. The choice for the next Task is seperated in three parts:
/// 1.) There is a `READY` Task in the `TASKS` vector -> schedule this task next.
/// 2.) Else, the top task `sleep_ticks` are smaller then the actuall timestamp_counter -> schedule
/// 3.) Else, no task is ready to run -> schedule `Idle` task, respectively, keep `Idle` as running
/// if `Idle` was the last running task.
///
/// # Arguments
/// * `f` - (ExceptionStackFrame) stores the data which are given by an interrupt, in this case by
/// a timer interrupt. the `ExceptionStackFrame` including the `cpu_flags`, `stack_pointer`,
/// `instruction_pointer`, and some other data which the scheduler doesn't use.
///
pub fn schedule(f: &mut ExceptionStackFrame) {
    //early_trace!();
    let cpuflags = f.cpu_flags;
    let stackpointer = f.stack_pointer;
    let instructionpointer = f.instruction_pointer;
    // check if a task is ready to run
    let tsc = rdtsc();
    let to_run = if TASKS.lock().last().expect("last").status == TaskStatus::READY {
        trace_debug!("popped ready task");
        let x = TASKS.lock().pop().expect("popped");
        x
    } else if TASKS.lock().last().expect("tadaa").sleep_ticks < tsc as usize {
        let x = TASKS.lock().pop().expect("popped");
        trace_debug!("popped after sleep task {}", x.sleep_ticks);
        x
    } else if unsafe { RUNNING_TASK.lock().status == TaskStatus::IDLE } {
        //early_trace!("do nothing");
        return;
    } else {
        trace_debug!("popped idle");
        let x = TASKS.lock().remove(0);
        x
    };
    //let to_run = TASKS.lock().pop().expect("scheduler schedule failed");
    //trace!("task: {}", to_run.instruction_pointer);

    unsafe {
        let not_finished = RUNNING_TASK.lock().status != TaskStatus::FINISHED;
        if not_finished {
            let name_c = RUNNING_TASK.lock().name;
            let pid_c = RUNNING_TASK.lock().pid;
            let sleep_ticks_c = RUNNING_TASK.lock().sleep_ticks;
            let time_sleep_c = RUNNING_TASK.lock().time_sleep;
            let time_active_c = RUNNING_TASK.lock().time_active;
            let last_time_stamp_c = RUNNING_TASK.lock().last_time_stamp;
            // PID = 0 --> main function
            //let old = TaskData::new(cpuflags, stackpointer, instructionpointer, to_run.status);
            let new_status = if RUNNING_TASK.lock().status == TaskStatus::IDLE {
                TaskStatus::IDLE
            } else {
                TaskStatus::RUNNING
            };
            let time_active_c = if (tsc as usize) < last_time_stamp_c {
                0
            } else {
                tsc as usize - last_time_stamp_c
            };
            let old = TaskData::copy(
                name_c,
                pid_c,
                cpuflags,
                stackpointer,
                instructionpointer,
                new_status,
                sleep_ticks_c,
                time_sleep_c,
                time_active_c,
                tsc as usize,
            );
            let mut position = 0;
            if old.status == TaskStatus::IDLE {
                TASKS.lock().insert(position, old);
            } else {
                for task in TASKS.lock().iter() {
                    if task.sleep_ticks <= sleep_ticks_c && task.status != TaskStatus::IDLE {
                        break;
                    }
                    position += 1;
                }
                TASKS.lock().insert(position, old);
            }
        }

        f.stack_pointer = to_run.stack_pointer;
        f.instruction_pointer = to_run.instruction_pointer;

        RUNNING_TASK.lock().name = to_run.name;
        RUNNING_TASK.lock().status = to_run.status;
        RUNNING_TASK.lock().stack_pointer = to_run.stack_pointer;
        RUNNING_TASK.lock().instruction_pointer = to_run.instruction_pointer;
        RUNNING_TASK.lock().pid = to_run.pid;
        RUNNING_TASK.lock().sleep_ticks = to_run.sleep_ticks;
        RUNNING_TASK.lock().time_sleep = tsc as usize - to_run.last_time_stamp;
        RUNNING_TASK.lock().time_active = to_run.time_active;
        RUNNING_TASK.lock().last_time_stamp = tsc as usize;
    }
}
