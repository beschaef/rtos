use alloc::Vec;
use features::msleep;
use scheduler::RUNNING_TASK;
use spin::Mutex;
use vga_buffer;
use vga_buffer::Color;
use x86_64::instructions::rdtsc;
use x86_64::VirtualAddress;

static mut PID_COUNTER: usize = 0;

const BOARD_WIDTH: u8 = 20; //20
const BOARD_HEIGHT: u8 = 17; //17
const ROW_OFFSET: u8 = 2;
const COL_OFFSET: u8 = 50;

lazy_static! {
    pub static ref PIECE: Mutex<Piece> = Mutex::new(Piece {
        oldx: (BOARD_WIDTH / 2) as i8,
        posx: (BOARD_WIDTH / 2) as i8,
        oldy: 0,
        posy: 0,
        color: Color::Green,
        oldshape: vec![vec![0]],
        shape: vec![vec![0]],
    });
    pub static ref BOARD: Mutex<Board> = Mutex::new(Board {
        cells: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    });
}

pub struct Piece {
    color: Color,
    posx: i8,
    posy: i8,
    oldx: i8,
    oldy: i8,
    oldshape: Vec<Vec<u8>>,
    shape: Vec<Vec<u8>>,
}

impl Piece {
    pub fn new_random_piece(&mut self) {
        self.oldx = (BOARD_WIDTH / 2) as i8;
        self.posx = (BOARD_WIDTH / 2) as i8;
        self.oldy = 0;
        self.posy = 0;
        //generate random numer between 0 and 6
        let i = rdtsc() % 7;

        match i {
            0 => {
                self.color = Color::Green;
                self.oldshape = vec![vec![1, 1], vec![1, 1]];
                self.shape = vec![vec![1, 1], vec![1, 1]];
            }
            1 => {
                self.color = Color::Brown;
                self.oldshape = vec![vec![0, 0, 1], vec![1, 1, 1], vec![0, 0, 0]];
                self.shape = vec![vec![0, 0, 1], vec![1, 1, 1], vec![0, 0, 0]];
            }
            2 => {
                self.color = Color::Blue;
                self.oldshape = vec![vec![1, 0, 0], vec![1, 1, 1], vec![0, 0, 0]];
                self.shape = vec![vec![1, 0, 0], vec![1, 1, 1], vec![0, 0, 0]];
            }
            3 => {
                self.color = Color::Cyan;
                self.oldshape = vec![vec![0, 1, 0], vec![1, 1, 1], vec![0, 0, 0]];
                self.shape = vec![vec![0, 1, 0], vec![1, 1, 1], vec![0, 0, 0]];
            }
            4 => {
                self.color = Color::Magenta;
                self.oldshape = vec![vec![0, 1, 1], vec![1, 1, 0], vec![0, 0, 0]];
                self.shape = vec![vec![0, 1, 1], vec![1, 1, 0], vec![0, 0, 0]];
            }
            5 => {
                self.color = Color::White;
                self.oldshape = vec![vec![1, 1, 0], vec![0, 1, 1], vec![0, 0, 0]];
                self.shape = vec![vec![1, 1, 0], vec![0, 1, 1], vec![0, 0, 0]];
            }
            6 => {
                self.color = Color::Yellow;
                self.oldshape = vec![
                    vec![0, 0, 0, 0],
                    vec![1, 1, 1, 1],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                ];
                self.shape = vec![
                    vec![0, 0, 0, 0],
                    vec![1, 1, 1, 1],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                ];
            }
            _ => println!("something else"),
        }
    }

    pub fn print_piece(&mut self) {
        self.each_old_point(&mut |row, col| {
            let oldx = self.oldx + col;
            let oldy = self.oldy + row;
            vga_buffer::write_at_background(
                " ",
                ROW_OFFSET + oldy as u8,
                COL_OFFSET + oldx as u8,
                Color::Black,
                Color::Black,
            );
        });

        self.each_point(&mut |row, col| {
            let posx = self.posx + col;
            let posy = self.posy + row;
            vga_buffer::write_at_background(
                "#",
                ROW_OFFSET + posy as u8,
                COL_OFFSET + posx as u8,
                self.color,
                Color::Black,
            );
        });
    }

    pub fn move_piece(&mut self, x: i8, y: i8) -> bool {
        let mut new_piece = Piece {
            oldx: self.posx,
            posx: self.posx + x,
            oldy: self.posy,
            posy: self.posy + y,
            color: self.color,
            oldshape: Vec::with_capacity(self.oldshape.len()),
            shape: Vec::with_capacity(self.shape.len()),
        };

        for row in &self.shape {
            new_piece.shape.push(row.clone());
            new_piece.oldshape.push(row.clone());
        }

        if new_piece.collision_test() {
            false
        } else {
            self.oldx = self.posx;
            self.posx = self.posx + x;
            self.oldy = self.posy;
            self.posy = self.posy + y;

            self.oldshape = Vec::with_capacity(self.shape.len());
            for row in &self.shape {
                self.oldshape.push(row.clone());
            }

            self.print_piece();

            true
        }
    }

    pub fn rotate(&mut self) {
        let size = self.shape.len();

        let mut new_piece = Piece {
            oldx: self.posx,
            posx: self.posx,
            oldy: self.posy,
            posy: self.posy,
            color: self.color,
            oldshape: Vec::with_capacity(self.oldshape.len()),
            shape: Vec::with_capacity(self.shape.len()),
        };
        new_piece.oldx = self.posx;
        new_piece.oldy = self.posy;
        for row in &self.shape {
            new_piece.shape.push(row.clone());
            new_piece.oldshape.push(row.clone());
        }

        for row in 0..size / 2 {
            for col in row..(size - row - 1) {
                let t = new_piece.shape[row][col];

                new_piece.shape[row][col] = new_piece.shape[col][size - row - 1];
                new_piece.shape[col][size - row - 1] =
                    new_piece.shape[size - row - 1][size - col - 1];
                new_piece.shape[size - row - 1][size - col - 1] =
                    new_piece.shape[size - col - 1][row];
                new_piece.shape[size - col - 1][row] = t;
            }
        }

        if !new_piece.collision_test() {
            self.oldx = self.posx;
            self.oldy = self.posy;
            self.oldshape = Vec::with_capacity(self.shape.len());
            for row in &self.shape {
                self.oldshape.push(row.clone());
            }

            for row in 0..size / 2 {
                for col in row..(size - row - 1) {
                    let t = self.shape[row][col];

                    self.shape[row][col] = self.shape[col][size - row - 1];
                    self.shape[col][size - row - 1] = self.shape[size - row - 1][size - col - 1];
                    self.shape[size - row - 1][size - col - 1] = self.shape[size - col - 1][row];
                    self.shape[size - col - 1][row] = t;
                }
            }

            self.print_piece();
        }
    }

    pub fn collision_test(&mut self) -> bool {
        let mut found = false;
        self.each_point(&mut |row, col| {
            if !found {
                let x = self.posx + col;
                let y = self.posy + row;
                if x < 0 || x >= (BOARD_WIDTH as i8) || y < 0 || y >= (BOARD_HEIGHT as i8)
                    || BOARD.lock().cells[y as usize][x as usize] != None
                {
                    found = true;
                }
            }
        });

        found
    }

    pub fn lock_piece(&mut self) {
        self.each_point(&mut |row, col| {
            let x = self.posx + col;
            let y = self.posy + row;
            BOARD.lock().cells[y as usize][x as usize] = Some(self.color);
        });
    }

    fn each_old_point(&self, callback: &mut FnMut(i8, i8)) {
        let piece_width = self.oldshape.len() as i8;
        for row in 0..piece_width {
            for col in 0..piece_width {
                if self.oldshape[row as usize][col as usize] != 0 {
                    callback(row, col);
                }
            }
        }
    }
    pub fn advance_game(&mut self) -> bool {
        if !self.move_piece(0, 1) {
            self.lock_piece();
            self.new_random_piece();
            if self.collision_test() {
                return false;
            }
            BOARD.lock().clear_lines();
        }
        true
    }
    fn each_point(&self, callback: &mut FnMut(i8, i8)) {
        let piece_width = self.shape.len() as i8;
        for row in 0..piece_width {
            for col in 0..piece_width {
                if self.shape[row as usize][col as usize] != 0 {
                    callback(row, col);
                }
            }
        }
    }
}

pub struct Board {
    cells: [[Option<Color>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Board {
    pub fn render_board(&self) {
        for y in 0..BOARD_HEIGHT {
            vga_buffer::write_at_background(
                "|",
                ROW_OFFSET + y as u8,
                COL_OFFSET - 1 as u8,
                Color::Red,
                Color::Red,
            );
            vga_buffer::write_at_background(
                "|",
                ROW_OFFSET + y as u8,
                COL_OFFSET + BOARD_WIDTH as u8,
                Color::Red,
                Color::Red,
            );
        }
        for x in 0..BOARD_WIDTH + 2 {
            vga_buffer::write_at_background(
                "-",
                ROW_OFFSET + BOARD_HEIGHT as u8,
                COL_OFFSET + x - 1 as u8,
                Color::Red,
                Color::Red,
            );
            //vga_buffer::write_at("-", ROW_OFFSET - 1, COL_OFFSET + x, Color::Red);
        }
    }

    pub fn clear_lines(&mut self) {
        for row_to_check in (0..BOARD_HEIGHT as usize).rev() {
            while !self.cells[row_to_check].iter().any(|x| *x == None) {
                print!("!");
                for row in (1..row_to_check + 1).rev() {
                    self.cells[row] = self.cells[row - 1];
                    for col in 0..BOARD_WIDTH as usize {
                        if self.cells[row][col] != None {
                            vga_buffer::write_at_background(
                                "#",
                                ROW_OFFSET + row as u8,
                                COL_OFFSET + col as u8,
                                self.cells[row][col].unwrap(),
                                Color::Black,
                            );
                        } else {
                            vga_buffer::write_at_background(
                                " ",
                                ROW_OFFSET + row as u8,
                                COL_OFFSET + col as u8,
                                Color::Black,
                                Color::Black,
                            );
                        }
                        vga_buffer::write_at_background(
                            " ",
                            ROW_OFFSET + row as u8 - 1,
                            COL_OFFSET + col as u8,
                            Color::Black,
                            Color::Black,
                        );
                    }
                }
            }
        }
    }
}

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
        let text = &format!(
            "{:2}:{:2}:{:2}",
            (r / (60 * 60)) % 24,
            (r / (60)) % 60,
            r % (60)
        );
        vga_buffer::write_at_background(text, 0, 0, color, Color::Black);
        early_trace!("Uptime1 written {:?}", text);
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
        let text = &format!(
            "{:2}:{:2}:{:2}",
            (r / (60 * 60)) % 24,
            (r / (60)) % 60,
            r % (60)
        );
        vga_buffer::write_at_background(text, 2, 0, color, Color::Black);
        early_trace!("Uptime2 written {:?}", text);
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
        let text = &format!(
            "{:2}:{:2}:{:2}",
            (r / (60 * 60)) % 24,
            (r / (60)) % 60,
            r % (60)
        );
        vga_buffer::write_at_background(text, 4, 0, color, Color::Black);
        early_trace!("Uptime3 written {:?}", text);
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
        let text = &format!(
            "{:2}:{:2}:{:2}",
            (r / (60 * 60)) % 24,
            (r / (60)) % 60,
            r % (60)
        );
        vga_buffer::write_at_background(text, 6, 0, color, Color::Black);
        early_trace!("Uptime4 written {:?}", text);
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
        let text = &format!(
            "{:2}:{:2}:{:2}",
            (r / (60 * 60)) % 24,
            (r / (60)) % 60,
            r % (60)
        );
        vga_buffer::write_at_background(text, 8, 0, color, Color::Black);
        early_trace!("Uptime5 written {:?}", text);
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
        let text = &format!(
            "{:2}:{:2}:{:2}",
            (r / (60 * 60)) % 24,
            (r / (60)) % 60,
            r % (60)
        );
        vga_buffer::write_at_background(text, 10, 0, color, Color::Black);
        early_trace!("Uptime_temp written {:?}", text);
        msleep(1000);
    }
    finish_task();
}

pub fn tetris() {
    msleep(1000);
    early_trace!();
    let mut gameover = false;
    PIECE.lock().new_random_piece();
    BOARD.lock().render_board();
    PIECE.lock().print_piece();

    while !gameover {
        PIECE.lock().print_piece();
        msleep(1000);
        if !PIECE.lock().advance_game() {
            gameover = true;
            msleep(1000);
        }
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
