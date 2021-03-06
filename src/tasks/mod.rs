//! This module manages everything related to tasks. It provides a common structure for generating new tasks.
//! The *callback* functions of all scheduled tasks are defined here. So if developers want to enhance the system with
//! new tasks, the function which is called by the scheduler when executing the task should be defined here.
//! Specific functions used by a task should be defined in external modules/crates and should only be called from here.
//! This module contains also some helper functions to manage tasks, e.g. `increment_pid()` or `finish_task()`.


use alloc::string::String;
use alloc::Vec;
use features::keyboard;
use features::{msleep, shell::*, test_bit};
use scheduler::RUNNING_TASK;
use scheduler::TASKS;
use spin::Mutex;
use vga_buffer;
use vga_buffer::Color;
use x86_64;
use x86_64::instructions::rdtsc;
use x86_64::VirtualAddress;

/// Stores the highest process id.
static mut PID_COUNTER: usize = 0;

/// Tells the scheduler if a task was started by the shell.
pub static mut TASK_STARTED: bool = false;

/// Stores the highscore from tetris.
static mut HIGHSCORE: usize = 0;

/// Sets the width of the tetris playing field.
const BOARD_WIDTH: u8 = 20;
/// Sets the height of the tetris  playing field.
const BOARD_HEIGHT: u8 = 17;

/// Sets the y-position of the tetris playing field (distance to left/top corner).
const ROW_OFFSET: u8 = 2;
/// Sets the x-position of the tetris playing field (distance to left/top corner).
const COL_OFFSET: u8 = 50;

lazy_static! {
/// The currently falling piece
    pub static ref PIECE: Mutex<Piece> = Mutex::new(Piece {
        //The previous position of the piece
        oldx: (BOARD_WIDTH / 2) as i8,
        posx: (BOARD_WIDTH / 2) as i8,
        //The current position
        oldy: 0,
        posy: 0,
        //The color
        color: Color::Green,
        //The previous shape of the piece
        oldshape: vec![vec![0]],
        //The current shape of the piece
        shape: vec![vec![0]],
    });
    /// Global board, contains the occupied cells.
    pub static ref BOARD: Mutex<Board> = Mutex::new(Board {
        cells: [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
    });
    /// Vector to store new tasks. New tasks can be started by the shell or other tasks. The main task
    /// starts new tasks from this vector.
    pub static ref NEW_TASKS: Mutex<Vec<VirtualAddress>> = Mutex::new(vec![]);

    /// The global shell object
    pub static ref SHELL: Mutex<Shell> = Mutex::new(Shell::new((21, 11)));
}

/// Struct of the currently falling piece
pub struct Piece {
    /// The Color of the specific piece
    color: Color,
    /// The current position of the piece in x-direction
    posx: i8,
    /// The current position of the piece in y-direction
    posy: i8,
    /// The previous position of the piece in x-direction
    oldx: i8,
    /// The previous position of the piece in y-direction
    oldy: i8,
    /// The previous shape of the piece
    oldshape: Vec<Vec<u8>>,
    /// The current shape of the piece
    shape: Vec<Vec<u8>>,
}

impl Piece {
    /// Change the current piece to a random kind of piece
    /// and put it at the top of the playing field.
    pub fn new_random_piece(&mut self) {
        self.oldx = (BOARD_WIDTH / 2) as i8;
        self.posx = (BOARD_WIDTH / 2) as i8;
        self.oldy = 0;
        self.posy = 0;

        // Generate random number between 0 and 6.
        let i = rdtsc() % 7;

        // Different piece shapes
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
            _ => println!("error creating new random piece"),
        }
    }

    /// Parses arrow keys to control the current falling piece.
    ///
    /// # Arguments
    ///
    /// * `control` - (String) Control command
    pub fn parse_control(&mut self, control: String) {
        if control == "ARROW_UP" {
            self.rotate();
        } else if control == "ARROW_DOWN" {
            self.advance_game();
        } else if control == "ARROW_LEFT" {
            self.move_piece(-1, 0);
        } else if control == "ARROW_RIGHT" {
            self.move_piece(1, 0);
        }
    }

    /// Prints the currently falling piece.
    pub fn print_piece(&mut self) {
        // delet the previos printed piece
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

        // Prints the currently piece.
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

    /// Checks if the piece can move in the given direction.
    ///
    /// # Arguments
    ///   * `x` - (i8) Moves the piece to the side. Positive values for right shift, negative values for left shift.
    ///   * `y` - (i8) Moves the piece down. Shouldn't be a negative value.
    ///
    /// # Return
    ///  * `false` - (bool) If there would be a crash.
    ///  * `true` -  (bool) If there wouldn't be a crash. Also moves the current piece.
    ///
    pub fn move_piece(&mut self, x: i8, y: i8) -> bool {
        // Clone the piece
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

        // Checks if there would be a collision.
        if new_piece.collision_test() {
            false
        } else {
            // move the piece
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

    /// Rotates the current piece counterclockwise.
    pub fn rotate(&mut self) {
        let size = self.shape.len();

        //Clone piece
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

        //Rotate the "clone"
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

        // Check if rotation would crash.
        // If it would crash ignores,
        // otherwise rotates the current piece.
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

    /// Checks if the piece would crash against the boarders of the playing field or against an occupied cell.
    ///
    /// # Return
    ///
    ///  * `true` - (bool) If there would be a crash.
    ///  * `false` - (bool) If there wouldn't be a crash.
    ///
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

    /// Add the falling piece to the occupied cells.
    pub fn lock_piece(&mut self) {
        self.each_point(&mut |row, col| {
            let x = self.posx + col;
            let y = self.posy + row;
            BOARD.lock().cells[y as usize][x as usize] = Some(self.color);
        });
    }

    /// Moves the current piece one step down.
    /// If this is not possible, locks the piece and
    /// creates a new random piece at the top of the field.
    /// If this is not possible as well, returns false.
    ///
    /// # Return
    ///
    ///  * `true` - (bool) If the piece is moved down or a new piece could be created.
    ///  * `false` - (bool) If the the piece couldn't be moved down and no new piece could be created -> game over.
    ///
    pub fn advance_game(&mut self) -> bool {
        if !self.move_piece(0, 1) {
            self.lock_piece();
            if BOARD.lock().cells[(ROW_OFFSET) as usize][(BOARD_WIDTH / 2) as usize] != None {
                vga_buffer::write_at_background(
                    &format!("- GAME OVER - HS: {} ", unsafe { HIGHSCORE }),
                    ROW_OFFSET - 2,
                    COL_OFFSET - 1,
                    Color::Red,
                    Color::Black,
                );
                unsafe {
                    HIGHSCORE = 0;
                }
                return false;
            }
            self.new_random_piece();
            if self.collision_test() {
                vga_buffer::write_at_background(
                    &format!("- GAME OVER - HS: {} ", unsafe { HIGHSCORE }),
                    ROW_OFFSET - 2,
                    COL_OFFSET - 1,
                    Color::Red,
                    Color::Black,
                );
                unsafe {
                    HIGHSCORE = 0;
                }
                return false;
            }
            BOARD.lock().clear_lines();
        }
        true
    }

    /// Returns each occupied cell of the shape.
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

    /// Returns each occupied cell of the previous shape.
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
}

/// Struct of the global board, contains the occupied cells.
pub struct Board {
    cells: [[Option<Color>; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize],
}

impl Board {
    /// Prints the boarders of the playing field.
    pub fn render_board(&mut self) {
        self.cells = [[None; BOARD_WIDTH as usize]; BOARD_HEIGHT as usize];
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
        }
        vga_buffer::write_at_background(
            &format!("Highscore: "),
            ROW_OFFSET - 2,
            COL_OFFSET - 1,
            Color::White,
            Color::Black,
        );
    }

    /// Clears each line that is filled completely.
    pub fn clear_lines(&mut self) {
        for row_to_check in (0..BOARD_HEIGHT as usize).rev() {
            while !self.cells[row_to_check].iter().any(|x| *x == None) {
                vga_buffer::write_at_background(
                    &format!("{:4}", increment_highscore()),
                    ROW_OFFSET - 2,
                    COL_OFFSET + 10,
                    Color::White,
                    Color::Black,
                );
                for row in (1..row_to_check + 1).rev() {
                    self.cells[row] = self.cells[row - 1];
                    for col in 0..BOARD_WIDTH as usize {
                        //transfer the line above to the current
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
                        // Clear the line above
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

/// Used to represent the current task status. In this system four different status are used.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// Used for the idle task
    IDLE,
    /// Used for a new task which has never run before
    READY,
    /// Used for all active tasks in the system
    RUNNING,
    /// Used when a task is terminated to show the scheduler that this task can be removed from the
    /// task list.
    FINISHED,
}

/// Stores all relevant data of a task. Each task gets an own `TaskData`.
#[derive(Debug, Clone)]
pub struct TaskData {
    /// For simplification only a char is used to represent the name of the task. Strings are causing
    /// problems due to allocation and ownership.
    pub name: char,
    /// Identification of a task. The main Task starts by 1, each new task will increment this value
    /// by 1.
    pub pid: usize,
    /// Stores the `cpu_flags` for scheduling.
    pub cpu_flags: u64,
    /// Stores the `stack_pointer` for scheduling.
    pub stack_pointer: VirtualAddress,
    /// Stores the `instruction_pointer` for scheduling.
    pub instruction_pointer: VirtualAddress,
    /// Stores the `TaskStatus` to show the scheduler the status of a task
    pub status: TaskStatus,
    /// Saves a timestamp. The task sleeps until this timestamp.
    pub sleep_ticks: usize,
    /// Used for logging / `htop`. Stores the time the task slept.
    pub time_sleep: usize,
    /// Used for logging / `htop`. Stores the time the task was active.
    pub time_active: usize,
    /// Used for logging / `htop`. Stores a delta value for calculation.
    pub last_time_stamp: usize,
}

impl TaskData {
    /// Creates a new `TaskData`.
    ///
    /// # Arguments
    /// * `name` - (char) *Name* of the taks. Currently only a char (see description above).
    /// * `cpu_flags` - (u64) cpu flags.
    /// * `stack_pointer` - (VirtualAddress)
    /// * `instruction_pointer` - (VirtualAddress)
    /// * `status` - (TaskStatus)
    ///
    /// # Return
    /// * TaskData - New created `TaskData`.
    pub fn new(
        name: char,
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        status: TaskStatus,
    ) -> Self {
        TaskData {
            name: name,
            pid: increment_pid(),
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            status,
            sleep_ticks: 0,
            time_sleep: 1,
            time_active: 1,
            last_time_stamp: 1,
        }
    }

    /// Copies a `TaskData` and returns a new `TaskData` object.
    pub fn copy(
        name: char,
        pid: usize,
        cpu_flags: u64,
        stack_pointer: VirtualAddress,
        instruction_pointer: VirtualAddress,
        status: TaskStatus,
        sleep_ticks: usize,
        time_sleep: usize,
        time_active: usize,
        last_time_stamp: usize,
    ) -> Self {
        TaskData {
            name,
            pid,
            cpu_flags,
            stack_pointer,
            instruction_pointer,
            status,
            sleep_ticks,
            time_sleep,
            time_active,
            last_time_stamp,
        }
    }
}

/// Clock which counts every second. The clock is starting by 00:00:00 on the top left corner.
/// This function increments a variable by one each time and then calcutes the seconds / minutes / hours.
pub fn uptime1() {
    msleep(1000);
    trace_info!();

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
        trace_debug!("Uptime1 written {:?}", text);
        msleep(1000);
    }
}

/// Similar to `uptime1()`, but on row 2.
pub fn uptime2() {
    msleep(1000);
    trace_info!();
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
        trace_debug!("Uptime2 written {:?}", text);
        msleep(1000);
    }
}

/// Similar to `uptime1()`, but on row 4.
pub fn uptime3() {
    msleep(1000);
    trace_info!();
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
        trace_debug!("Uptime3 written {:?}", text);
        msleep(1000);
    }
}

/// Similar to `uptime1()`, but on row 6.
pub fn uptime4() {
    msleep(1000);
    trace_info!();
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
        trace_debug!("Uptime4 written {:?}", text);
        msleep(1000);
    }
}

/// Used to add frequently new temporary clocks. Only use this function for testing. Otherwise the
/// system will run out of heap due to frequently allocatin new heap.
#[allow(dead_code)]
pub fn add_new_temp_clocks() {
    msleep(2000);
    trace_info!();
    loop {
        msleep(1000);
        NEW_TASKS
            .lock()
            .insert(0, VirtualAddress(uptime_temp as usize));
        msleep(10000);
        trace_debug!("Added new temp task");
    }
}

/// Temporary clock to show that it is possibly to start and stop tasks while the system is running.
/// The clock counts - similar to the other clock tasks - but only up to 10.
pub fn uptime_temp() {
    msleep(1000);
    trace_info!();
    let mut r = 0;
    for _i in 0..10 {
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
        trace_debug!("Uptime_temp written {:?}", text);
        msleep(1000);
    }
    vga_buffer::write_at_background("          ", 10, 0, Color::Black, Color::Black);
    finish_task();
}

/// Task of the tetris game.
pub fn tetris() {
    msleep(1000);
    trace_info!();
    let mut gameover = false;

    // Print boarders of the playing field.
    BOARD.lock().render_board();

    // Create first random piece
    PIECE.lock().new_random_piece();

    // While the game is not over
    while !gameover {
        if SHELL.lock().terminate_running_task == true {
            SHELL.lock().reset_shell();
            // end the task
            finish_task();
        }

        PIECE.lock().print_piece();
        // New piece every second
        msleep(1000);

        // Game over if the game couldn't be continued
        if !PIECE.lock().advance_game() {
            gameover = true;
            msleep(1000);
        }
    }
    msleep(2000);
    SHELL.lock().reset_shell();
    finish_task();
}

/// Task of the shell.
pub fn shell() {
    msleep(1500);
    SHELL.lock().init_shell();
    loop {
        unsafe {
            if TASK_STARTED != true {
                SHELL.lock().cursor_on();
                msleep(500);
                SHELL.lock().cursor_off();
                msleep(500);
            } else {
                msleep(100);
            }
        }
    }
    //    finish_task();
}

/// Task of htop.
/// Prints out all the active tasks and computes their utilization.
/// The utilization results from the active time divided by the active + passive time.
/// The process is looping permanently while the processes are calculated and printed. There are no interrupts allowed to avoid concurrency problems
pub fn htop() {
    trace_info!();
    loop {
        msleep(1000);
        unsafe {
            x86_64::instructions::interrupts::disable();
        }
        for (i, task) in TASKS.lock().iter().enumerate() {
            let percent_digits = calc_float_percent_from_int(
                task.time_active,
                task.time_active + task.time_sleep,
                4,
            );
            let name = format!(
                "Task {}: {}{}.{}{}%",
                task.name,
                percent_digits[0],
                percent_digits[1],
                percent_digits[2],
                percent_digits[3]
            );
            //delete next line
            vga_buffer::write_at_background(
                "                    ",
                i as u8 + 1,
                15,
                Color::Black,
                Color::Black,
            );
            vga_buffer::write_at_background(&name, i as u8, 15, Color::Red, Color::Black);
        }

        unsafe {
            x86_64::instructions::interrupts::enable();
        }
    }

    // End the task, finish_task is unreachable
    // finish_task();
}

/// Calculates the digits of the percentage ratio of two usize numbers, of which a float can be
/// built.
///
/// # Arguments
/// * `num` - (usize) Numerator
/// * `denom` - (usize) Denominator
/// * `digits` - (usize) Total number of digits of the resulting percentage ratio
///
/// # Return
/// * `percent_digits` - (Vec<usize>) Resulting digits of percentage ratio
///
/// # Example
/// * nom - 1
/// * denom - 8
/// * digits - 3
/// This example will return a vector [1, 2, 5] (12.5 %)
fn calc_float_percent_from_int(num: usize, denom: usize, digits: usize) -> Vec<usize> {
    let mut percent_digits: Vec<usize> = Vec::new();
    let mut int = num * 10_usize.pow(digits as u32) / denom;
    let mut cnt = digits;
    for _i in 0..digits {
        cnt -= 1;
        let digit = int / 10_usize.pow(cnt as u32);
        percent_digits.push(digit);
        int = int - (digit * 10_usize.pow(cnt as u32));
    }
    percent_digits
}

/// Moved keyboard handler from interrupts to own function
/// keyboard handler as interrupt causes to PIC's deadlock problems.
/// for now the function polls every 50ms.
///
/// https://wiki.osdev.org/PS/2_Keyboard
///
/// If bit 1 of the byte on port `0x64` is set, there is some user input. With bit 5 of this byte one can
/// distinguish between mouse and keyboard events. Finally the function reads the scancode on port `0x60` to
/// get the actual input.
use x86_64::instructions::port;
pub fn task_keyboard() {
    msleep(1000);
    unsafe {
        loop {
            let user_input = port::inb(0x64);
            if test_bit(user_input, 0x1) {
                // general user input event
                if !test_bit(user_input, 0x20) {
                    // if bit 5 is set -> mouse event
                    let scan_code = port::inb(0x60);
                    if let Some(c) = keyboard::from_scancode(scan_code as usize) {
                        SHELL.lock().parse_input(c);
                    }
                }
            }
            msleep(20);
        }
    }
}

/// Idle Task, only running when no other task is ready. This function needs inline assemby to bring
/// the cpu into the pause mode and not waste cpu.
pub fn idle_task() {
    trace_info!("IDLE");
    loop {
        unsafe {
            asm!("pause":::: "intel", "volatile");
        }
    }
}

/// Each new task is getting an unique process ID. This function simply increments the PID_COUNTER by one
/// and then returns the new PID.
fn increment_pid() -> usize {
    unsafe {
        PID_COUNTER += 1;
        PID_COUNTER
    }
}

/// Similar to the function `increment_pid()`, this function increments the tetris highscore by one.
fn increment_highscore() -> usize {
    unsafe {
        HIGHSCORE += 1;
        HIGHSCORE
    }
}

/// Always called at the end of a task. The running task is marked as finished. After that the
/// scheduler is called by a timer interrupt.
fn finish_task() {
    trace_info!("TASK FINISHED");
    unsafe {
        RUNNING_TASK.lock().status = TaskStatus::FINISHED;
        int!(0x20);
    }
}
