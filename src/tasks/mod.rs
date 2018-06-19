use vga_buffer;
use vga_buffer::Color;
use features::msleep;
use x86_64::VirtualAddress;
use scheduler::RUNNING_TASK;
use alloc::String;

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
        vga_buffer::write_at_background(text, 0, 0, color, Color::Black);
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
        vga_buffer::write_at_background(text, 2, 0, color, Color::Black);
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
        vga_buffer::write_at_background(text, 4, 0, color, Color::Black);
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
        vga_buffer::write_at_background(text, 6, 0, color, Color::Black);
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
        vga_buffer::write_at_background(text, 8, 0, color, Color::Black);
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
        vga_buffer::write_at_background(text, 10, 0, color, Color::Black);
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

    const BOARD_WIDTH: u8 = 20;
    const BOARD_HEIGHT: u8 = 15;
    const ROW_OFFSET: u8 = 3;
    const COL_OFFSET: u8 = 40;


    struct Board {
        cells: [[Option<Color>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    }

    impl Board {
        pub fn render_board(&self) {
            for y in 0..BOARD_HEIGHT {
                vga_buffer::write_at_background("|", ROW_OFFSET + y as u8, COL_OFFSET -1 as u8, Color::Red, Color::Black);
                vga_buffer::write_at_background("|", ROW_OFFSET + y as u8, COL_OFFSET + BOARD_WIDTH + 1 as u8, Color::Red, Color::Black);
            }
            for x in 0..BOARD_WIDTH {
                vga_buffer::write_at_background("-", ROW_OFFSET + BOARD_HEIGHT as u8, COL_OFFSET + x as u8, Color::Red, Color::Black);
                //vga_buffer::write_at("-", ROW_OFFSET - 1, COL_OFFSET + x, Color::Red);
            }

        }
    }
    struct Piece {
        color: Color,
        posx: i8,
        posy: i8,
        oldx: i8,
        oldy: i8,
        shape: String,
    }

    impl Piece {
        pub fn print_piece(&mut self){
            vga_buffer::write_at_background(" ", ROW_OFFSET + self.oldy as u8, COL_OFFSET +self.oldx as u8, Color::Black, Color::Black);
            vga_buffer::write_at_background(&self.shape, ROW_OFFSET + self.posy as u8, COL_OFFSET +self.posx as u8, self.color, Color::Black);
        }

        pub fn move_piece(&mut self, board: &Board, x: i8, y: i8) -> bool{
            let mut new_piece = Piece{
                oldx: self.posx,
                posx: self.posx + x,
                oldy: self.posy,
                posy: self.posy + y,
                color: self.color,
                shape: String::from("#"),
            };

            if new_piece.collision_test(board){
                false
            }else{
                self.oldx= self.posx;
                self.posx= self.posx + x;
                self.oldy= self.posy;
                self.posy= self.posy + y;

                true
            }
        }

        pub fn collision_test(&mut self, board: &Board) -> bool {
            let mut found = false;

            if self.posx < 0 || self.posx >= (BOARD_WIDTH as i8) || self.posy < 0 || self.posy >= (BOARD_HEIGHT as i8) ||
                (board.cells[self.posy as usize][self.posx as usize] != None) {
                found = true;
            }
            found
        }

        pub fn lock_piece(&mut self, board: &mut Board) {
            board.cells[self.posy as usize][self.posx as usize] = Some(self.color);
        }
    }



    loop {
        let mut board = Board{
            cells: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize]
        };

        let mut piece = Piece{
            color: Color::Green, shape: String::from("#"),posx:(BOARD_WIDTH/2) as i8, posy: 0, oldx:0, oldy:0
        };

        board.render_board();
        piece.print_piece();

        loop{
            msleep(2000);
            if !piece.move_piece(&board, 0, 1){
                piece.lock_piece(&mut board);
                piece = Piece{
                    color: Color::Green, shape: String::from("#"),posx:(BOARD_WIDTH/2) as i8, posy: 0, oldx:0, oldy:0
                };
            }
            piece.print_piece();
        }
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