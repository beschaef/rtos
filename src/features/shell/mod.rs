//! This module provides a basic shell / terminal to enable user interaction with the system.
//! It supports a minimal set of commands and is easily enhancable to implement new commands.
//! Currently supported commands are:
//!
//!     1. "help"     -> Displays a man page which shows a list of supported commands.
//!     2. "tetris"   -> Starts a self developed version of the ancient tetris
//!     3. "clock"    -> Adds a temporary clock which counts up to a specific amount of seconds
//!     4. "reboot"   -> Reboots the system
//!     5. "shutdown" -> Shuts down the system
//!     6. "strg + c" -> Terminates the current running task which was issued from the shell
//!
//! A new Shell can be initialized with a custom number of lines, which is determined by passing
//! the initial cursor position (altogether 25 rows are available on the screen).
//! ATTENTION: Currently all screen output not coming from the shell is not adapted to the area
//! which is occupied by the shell, which means that there can be overlaps (dependent on the initial
//! cursor position).

use alloc::string::String;
use alloc::{string::ToString, Vec};
use features::{reboot, shutdown};
use tasks::{tetris, uptime_temp, NEW_TASKS, PIECE, TASK_STARTED};
#[allow(unused_imports)]
use trace::*;
use vga_buffer::*;
use x86_64::VirtualAddress;

pub struct Shell {
    /// Specifies the initial cursor position (row, col).
    default_cursor_position: (u8, u8),
    /// Specifies the current cursor position (row, col). Values change during runtime.
    current_cursor_position: (u8, u8),
    /// Stores the user input from the shell.
    input: String,
    /// Stores the commands issued from the shell.
    input_history: Vec<String>,
    /// If set to true, the currently running task terminates itself and is not scheduled anymore.
    pub terminate_running_task: bool,
    /// If set to true, the shell checks whether ctrl + c was pressed.
    parse_ctrl_command: bool,
    /// Contains the running task (started by the shell) as string.
    running_task: String,
    /// Defines the screen area to which the content of tasks started by the shell is displayed.
    active_screen: (u8, u8, u8, u8),
    /// Text, which is displayed when the user issues an unsupported command.
    unkown_command_help: String,
}

impl Shell {
    /// Creates a new shell and sets some default default values.
    /// # Arguments
    /// * `current_cursor_position` - ((u8, u8)) Specifies the initial cursor position (row, col)
    /// and therewith defines the number of lines occupied by the shell.
    #[allow(dead_code)]
    pub fn new(current_cursor_position: (u8, u8)) -> Self {
        Shell {
            default_cursor_position: current_cursor_position,
            current_cursor_position: current_cursor_position,
            input: "".to_string(),
            input_history: Vec::new(),
            terminate_running_task: false,
            parse_ctrl_command: false,
            active_screen: (30, 80, 0, 20),
            running_task: "".to_string(),
            unkown_command_help:
                "Unknown command. To see a list of possible commands, type `help`!".to_string(),
        }
    }

    /// Initializes the shell -> Prints the line which seperates the active screen from the shell
    /// area and prints the prompt to the according position.
    pub fn init_shell(&mut self) {
        self.print_separating_line();
        let cursor_position_height = self.current_cursor_position.0;
        self.print_prompt(cursor_position_height, 0);
    }

    /// Clears the active screen area (sets it to black fore- and background-color). Normally used
    /// when a task issued by the shell is terminated.
    pub fn clear_active_screen(&mut self) {
        for col in self.active_screen.0..self.active_screen.1 {
            for row in self.active_screen.2..self.active_screen.3 {
                write_at_background(" ", row, col, Color::Black, Color::Black);
            }
        }
    }

    /// Prints the line which seperates the active screen from the shell area.
    pub fn print_separating_line(&mut self) {
        write_at_background(
            "--------------------------------------------------------------------------------",
            self.current_cursor_position.0 - 1,
            0,
            Color::White,
            Color::Black,
        );
    }

    /// Prints the prompt to a desired position.
    /// # Arguments
    /// * `cursor_position_height` - (u8) Specifies the desired vertical position
    /// * `cursor_position_width` - (u8) Specifies the desired horizontal position
    pub fn print_prompt(&mut self, cursor_position_height: u8, cursor_position_width: u8) {
        write_at_background(
            "bob@rtos > ",
            cursor_position_height,
            cursor_position_width,
            Color::LightGray,
            Color::Black,
        );
    }

    /// Lets the cursor appear on the screen.
    /// Used together with cursor_off() to let the cursor blink.
    pub fn cursor_on(&mut self) {
        write_at_background(
            "_",
            self.current_cursor_position.0,
            self.current_cursor_position.1,
            Color::White,
            Color::Black,
        );
    }

    /// Lets the cursor disappear from the screen.
    /// Used together with cursor_on() to let the cursor blink.
    pub fn cursor_off(&mut self) {
        write_at_background(
            " ",
            self.current_cursor_position.0,
            self.current_cursor_position.1,
            Color::Black,
            Color::Black,
        );
    }

    /// Receives preprocessed user input from `parse_input()`.
    /// If currently no other task is running, the input is pushed to the string which
    /// contains all of the user input of the current line. Otherwise user input is disabled, which
    /// means that no user input is stored. After saving the input it is printed on the screen.
    /// Finally the cursor is shifted to the next input position.
    /// # Arguments
    /// * `input` - (String) Symbol corresponding to the pressed key on the keyboard, casted as
    /// string
    pub fn store_and_print_input(&mut self, input: String) {
        unsafe {
            if TASK_STARTED != true {
                self.input.push(input.chars().next().unwrap());
                write_at_background(
                    &input,
                    self.current_cursor_position.0,
                    self.current_cursor_position.1,
                    Color::White,
                    Color::Black,
                );
                if input.len() as u8 > 1 {
                    self.current_cursor_position.1 += input.len() as u8 + 1;
                } else if input.len() as u8 == 1 {
                    self.current_cursor_position.1 += 1;
                }
            }
        }
    }

    /// Entry point of user input to the shell module.
    /// Every time the user presses a key on the keyboard, a corresponding string is passed to this
    /// function, which then handles possible cases:
    ///
    /// 1. *ENTER*     -> The blinking cursor is removed from the shell (to signalize that
    /// user input is disabled) and parse_command() is called.
    /// 2. *BACKSPACE* -> If no task (started by the shell) is running and if the cursor is in
    /// a valid position, the last input char is removed from the input string and the cursor is
    /// shifted to the left.
    /// 3. *CTRL*      -> An internal flag is set to signalize that a ctrl-key was pressed.
    /// 4. *ARROW*     -> If tetris is running, a string corresponding to the pressed arrow key is
    /// passed to the control parser of tetris.
    /// 5. *DEFAULT*   -> If none of the previous cases applies, the input is not interpreted as
    /// command but as *normal* input and is passed to `store_and_print_input()`.
    /// # Arguments
    /// * `input` - (String) Symbol corresponding to the pressed key on the keyboard, casted as
    /// string
    pub fn parse_input(&mut self, input: String) {
        if input == "ENTER" {
            // delete blinking cursor in current line
            write_at_background(
                " ",
                self.current_cursor_position.0,
                self.current_cursor_position.1,
                Color::Black,
                Color::Black,
            );
            self.parse_command();
        } else if input == "BACKSPACE" {
            unsafe {
                if TASK_STARTED != true {
                    if self.current_cursor_position.1 > self.default_cursor_position.1 {
                        self.input.pop();
                        write_at_background(
                            " ",
                            self.current_cursor_position.0,
                            self.current_cursor_position.1,
                            Color::Black,
                            Color::Black,
                        );
                        self.current_cursor_position.1 -= 1;
                    }
                }
            }
        } else if input == "CTRL_LEFT" || input == "CTRL_RIGHT" {
            self.parse_ctrl_command = true;
        } else if self.parse_ctrl_command == true {
            self.parse_ctrl_command(input);
        } else if input == "ARROW_UP" || input == "ARROW_DOWN" || input == "ARROW_LEFT"
            || input == "ARROW_RIGHT"
        {
            unsafe {
                if TASK_STARTED && self.running_task == "tetris" {
                    PIECE.lock().parse_control(input);
                }
            }
        } else {
            if self.current_cursor_position.1 < BUFFER_WIDTH as u8 - 1 {
                self.store_and_print_input(input);
            }
        }
    }

    /// Called by `parse_input()`.
    /// If neccessary (e.g. in case of tetris, clock), the function pointer of the task which should
    /// be started is pushed to the vector NEW_TASKS, which stores new tasksto be started by the shell or other tasks.
    /// The main task starts new task from this vector.
    /// In case of *reboot* or *shutdown* the corresponding function in the *features* crate is called.
    /// If an unsupported command is issued, an appropriate warning is displayed.
    fn parse_command(&mut self) {
        let x = self.input.to_string();
        self.input_history.push(x.clone());
        if x == "tetris" {
            {
                NEW_TASKS.lock().insert(0, VirtualAddress(tetris as usize));
            }
            unsafe {
                TASK_STARTED = true;
            }
            self.running_task = "tetris".to_string();
        } else if x == "htop" {
            ;
        } else if x == "help" {
            self.show_shell_manual();
            unsafe {
                TASK_STARTED = true;
            }
            self.running_task = "help".to_string();
        } else if x == "clock" {
            {
                NEW_TASKS
                    .lock()
                    .insert(0, VirtualAddress(uptime_temp as usize));
            }
            if self.current_cursor_position.0 as usize >= BUFFER_HEIGHT - 1 {
                self.print_shift_history();
            } else {
                self.current_cursor_position.0 += 1;
            }
            self.current_cursor_position.1 = self.default_cursor_position.1;
            let cursor_position_height = self.current_cursor_position.0;
            self.print_prompt(cursor_position_height, 0);
        } else if x == "" {
            ;
        } else if x == "reboot" {
            reboot();
        } else if x == "shutdown" {
            shutdown();
        } else {
            if self.current_cursor_position.0 as usize >= BUFFER_HEIGHT - 1 {
                clear_row(BUFFER_HEIGHT - 1);
                write_at_background(
                    &self.unkown_command_help,
                    BUFFER_HEIGHT as u8 - 1,
                    0,
                    Color::Red,
                    Color::Black,
                );
                {
                    let help = &self.unkown_command_help;
                    self.input_history.push(help.to_string());
                }
                self.print_shift_history();
                self.current_cursor_position.1 = self.default_cursor_position.1;
            } else {
                self.current_cursor_position.0 += 1;
                if self.current_cursor_position.0 as usize >= BUFFER_HEIGHT - 1 {
                    clear_row(BUFFER_HEIGHT - 1);
                    write_at_background(
                        &self.unkown_command_help,
                        BUFFER_HEIGHT as u8 - 1,
                        0,
                        Color::Red,
                        Color::Black,
                    );
                    {
                        let help = &self.unkown_command_help;
                        self.input_history.push(help.to_string());
                    }
                    self.print_shift_history();
                    self.current_cursor_position.1 = self.default_cursor_position.1;
                } else {
                    clear_row(self.current_cursor_position.0 as usize);
                    write_at_background(
                        &self.unkown_command_help,
                        self.current_cursor_position.0,
                        0,
                        Color::Red,
                        Color::Black,
                    );
                    {
                        let help = &self.unkown_command_help;
                        self.input_history.push(help.to_string());
                    }
                    self.current_cursor_position.0 += 1;
                    self.current_cursor_position.1 = self.default_cursor_position.1;
                    let cursor_position_height = self.current_cursor_position.0;
                    self.print_prompt(cursor_position_height, 0);
                }
            }
        }
        self.input.clear();
    }

    /// Prints a description of currently implemented shell commands.
    /// The text is written quick and dirty to the active screen area.
    /// A more professional solution would be to write a method which parses a desired textfile
    /// (possibly containing control commands for formatting) and renders the text to a specified
    /// area on the screen. Due to lack of time this was not realized any more.
    fn show_shell_manual(&mut self) {
        write_at_background(
            "###### RTOS-SHELL - MANUAL ######",
            0,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "1. help     > Shows a full list of possible",
            2,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "              shell commands",
            3,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "2. tetris   > Starts a funky tetris game",
            5,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "3. clock    > Adds a temporary clock to the",
            7,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "              left of the screen",
            8,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "4. reboot   > Reboots the system",
            10,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "5. shutdown > Powers off the system",
            12,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "6. ctrl-c   > Cancels the last command issued",
            14,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "              from the shell and activates",
            15,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "              new input",
            16,
            35,
            Color::White,
            Color::Black,
        );
    }

    /// Called by `parse_input()` after *ctrl* and another key was pressed.
    /// If the other key was *c* and a task started by the shell is running, this method sets the
    /// `terminate_running_task` flag to inform the scheduler that the task can be terminated.
    /// Then the input string is cleared and the shell is resetted to some default values.
    /// # Arguments
    /// * `input` - (String) Symbol corresponding to the pressed key on the keyboard, casted as string
    fn parse_ctrl_command(&mut self, input: String) {
        if input == "c" {
            if unsafe { TASK_STARTED } {
                if self.running_task == "help" {
                    self.reset_shell();
                } else {
                    self.terminate_running_task = true;
                }
                self.input.clear();
                self.running_task = "".to_string();
            }
        }
        self.parse_ctrl_command = false;
    }

    /// Sets the shell variables to some default values.
    /// For example the  `terminate_running_task` flag is cleared, the active screen area is cleared.
    /// If the last line is reached, the function `print_shift_history()` is called to shift the previous inputs
    /// up.
    pub fn reset_shell(&mut self) {
        self.terminate_running_task = false;
        self.clear_active_screen();
        if self.current_cursor_position.0 as usize >= BUFFER_HEIGHT - 1 {
            self.print_shift_history();
        } else {
            self.current_cursor_position.0 += 1;
        }
        self.current_cursor_position.1 = self.default_cursor_position.1;
        let cursor_position_height = self.current_cursor_position.0;
        self.print_prompt(cursor_position_height, 0);
        unsafe {
            TASK_STARTED = false;
        }
    }

    /// Called by several shell functions to shift the previous inputs up if the last line of the shell
    /// was reached.
    fn print_shift_history(&mut self) {
        let mut cnt: usize = 1;
        for _row in self.default_cursor_position.0 as usize..BUFFER_HEIGHT - 1 {
            clear_row(BUFFER_HEIGHT - 1 - cnt);
            let mut history_entry =
                &self.input_history[(self.input_history.len() - 1) - (cnt - 1)].to_string();
            if history_entry == &self.unkown_command_help {
                write_at_background(
                    history_entry,
                    BUFFER_HEIGHT as u8 - 1 - cnt as u8,
                    0,
                    Color::Red,
                    Color::Black,
                );
            } else {
                self.print_prompt(BUFFER_HEIGHT as u8 - 1 - cnt as u8, 0);
                write_at_background(
                    history_entry,
                    BUFFER_HEIGHT as u8 - 1 - cnt as u8,
                    self.default_cursor_position.1,
                    Color::White,
                    Color::Black,
                );
            }
            cnt += 1;
        }
        // clear last line and print a prompt
        clear_row(BUFFER_HEIGHT - 1);
        self.print_prompt(BUFFER_HEIGHT as u8 - 1, 0);
    }
}
