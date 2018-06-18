use vga_buffer;
use vga_buffer::Color;
use features::msleep;
use x86_64::VirtualAddress;
use scheduler::RUNNING_TASK;

static mut PID_COUNTER: usize = 0;

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    IDLE,
    READY,
    RUNNING,
    FINISHED,
}

#[derive(Debug, Clone)]
pub struct TaskData {
    pub pid: usize,
    pub cpu_flags: u64,
    pub stack_pointer: VirtualAddress,
    pub instruction_pointer: VirtualAddress,
    pub status: TaskStatus,
    pub sleep_ticks: usize,
}

///unsafe block is actually safe because we're initializing the tasks before the interrupts are enabled
impl TaskData {
    pub fn new(
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        status: TaskStatus,
    ) -> Self {
        TaskData {
            pid: increment_pid(),
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            status,
            sleep_ticks: 0,
        }
    }

    pub fn copy(
        pid: usize,
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        status: TaskStatus,
        sleep_ticks: usize,
    ) -> Self {
        TaskData {
            pid,
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            status,
            sleep_ticks,
        }
    }
}

pub fn uptime1() {
    msleep(1000);
    early_trace!();

    let mut r = 0;
    loop {

        r = r + 1;
        let color = Color::LightGreen;
        let text = &format!("{:2}:{:2}:{:2}",(r/(60*60))%24,(r/(60))%60,r%(60));
        vga_buffer::write_at(text, 0, 0, color);
        early_trace!("Uptime1 written {:?}",text);
        msleep(1000);
    }
}

pub fn uptime2() {
    msleep(1000);
    early_trace!();
    let mut r = 0;
    loop {
        //trace!("loop uptime1");

        r = r + 1;
        let color = Color::LightGreen;
        let text = &format!("{:2}:{:2}:{:2}",(r/(60*60))%24,(r/(60))%60,r%(60));
        vga_buffer::write_at(text, 2, 0, color);
        early_trace!("Uptime2 written {:?}",text);
        msleep(1000);
    }
}

pub fn uptime3() {
    msleep(1000);
    early_trace!();
    let mut r = 0;
    loop {
        //trace!("loop uptime1");

        r = r + 1;
        let color = Color::LightGreen;
        let text = &format!("{:2}:{:2}:{:2}",(r/(60*60))%24,(r/(60))%60,r%(60));
        vga_buffer::write_at(text, 4, 0, color);
        early_trace!("Uptime3 written {:?}",text);
        msleep(1000);
    }
}

pub fn uptime4() {
    msleep(1000);
    early_trace!();
    let mut r = 0;
    loop {
        //trace!("loop uptime1");

        r = r + 1;
        let color = Color::LightGreen;
        let text = &format!("{:2}:{:2}:{:2}",(r/(60*60))%24,(r/(60))%60,r%(60));
        vga_buffer::write_at(text, 6, 0, color);
        early_trace!("Uptime4 written {:?}",text);
        msleep(1000);
    }
}

#[allow(dead_code)]
pub fn uptime5() {
    msleep(2000);
    early_trace!();
    let mut r = 0;
    loop {
        //trace!("loop uptime1");

        r = r + 1;
        let color = Color::LightGreen;
        let text = &format!("{:2}:{:2}:{:2}",(r/(60*60))%24,(r/(60))%60,r%(60));
        vga_buffer::write_at(text, 8, 0, color);
        early_trace!("Uptime5 written {:?}",text);
        msleep(5000);
    }
}

pub fn uptime_temp() {
    msleep(1000);
    early_trace!();
    let mut r = 0;
    for _i in 0..3 {
        //trace!("loop uptime1");

        r = r + 1;
        let color = Color::LightGreen;
        let text = &format!("{:2}:{:2}:{:2}",(r/(60*60))%24,(r/(60))%60,r%(60));
        vga_buffer::write_at(text, 10, 0, color);
        early_trace!("Uptime_temp written {:?}",text);
        msleep(1000);
    }
    finish_task();
}

pub fn idle_task() {
    early_trace!("IDLE");
    loop {
        unsafe {
            asm!("pause":::: "intel", "volatile");
        }
    }
}

pub fn tetris() {
    msleep(1000);
    early_trace!();
    let mut r = 0;
    // let matrix = 40*40 array
    loop {
        //trace!("loop uptime1");

        let color = Color::LightGreen;
        let text = &format!("##");
        vga_buffer::write_at(text, 8, 50, color);
        let text = &format!("##");
        vga_buffer::write_at(text, 8, 50, color);

        early_trace!("Uptime5 written {:?}",text);
        msleep(200);
    }
}

fn increment_pid() -> usize {
    unsafe {
        PID_COUNTER += 1;
        PID_COUNTER
    }
}

fn finish_task() {
    early_trace!("TASK FINISHED");
    unsafe {
        RUNNING_TASK.lock().status = TaskStatus::FINISHED;
        int!(0x20);
    }
}