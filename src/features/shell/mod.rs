use alloc::string::String;
use alloc::{string::ToString, Vec};
use features::reboot;
use tasks::{tetris, uptime_temp, NEW_TASKS, PIECE, TASK_STARTED};
#[allow(unused_imports)]
use trace::*;
use vga_buffer::*;
use x86_64::VirtualAddress;

pub struct Shell {
    default_cursor_position: (u8, u8),
    current_cursor_position: (u8, u8),
    input: String,
    input_history: Vec<String>,
    pub terminate_running_task: bool,
    parse_ctrl_command: bool,
    running_task: String,
    active_screen: (u8, u8, u8, u8),
    unkown_command_help: String,
}

impl Shell {
    #[allow(dead_code)]
    pub fn new(current_cursor_position: (u8, u8)) -> Self {
        Shell {
            default_cursor_position: current_cursor_position,
            current_cursor_position: current_cursor_position,
            input: "".to_string(),
            input_history: Vec::new(),
            terminate_running_task: false,
            parse_ctrl_command: false,
            active_screen: (30,80,0,20),
            running_task: "".to_string(),
            unkown_command_help:
                "Unknown command. To see a list of possible commands, type `help`!".to_string(),
        }
    }

    pub fn init_shell(&mut self) {
        self.print_separating_line();
        let cursor_position_height = self.current_cursor_position.0;
        self.print_prompt(cursor_position_height, 0);
    }

    pub fn clear_active_screen(&mut self) {
        for col in self.active_screen.0..self.active_screen.1 {
            for row in self.active_screen.2..self.active_screen.3 {
                write_at_background(" ", row, col, Color::Black, Color::Black);
            }
        }
    }

    pub fn print_separating_line(&mut self) {
        write_at_background(
            "--------------------------------------------------------------------------------",
            self.current_cursor_position.0 - 1,
            0,
            Color::White,
            Color::Black,
        );
    }

    pub fn print_prompt(&mut self, cursor_position_height: u8, cursor_position_width: u8) {
        write_at_background(
            "bob@rtos > ",
            cursor_position_height,
            cursor_position_width,
            Color::LightGray,
            Color::Black,
        );
    }

    pub fn cursor_on(&mut self) {
        write_at_background(
            "_",
            self.current_cursor_position.0,
            self.current_cursor_position.1,
            Color::White,
            Color::Black,
        );
    }

    pub fn cursor_off(&mut self) {
        write_at_background(
            " ",
            self.current_cursor_position.0,
            self.current_cursor_position.1,
            Color::Black,
            Color::Black,
        );
    }

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
        } else if input == "BACKSPACE"{
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

    fn parse_command(&mut self) {
        let x = self.input.to_string();
        self.input_history.push(x.clone());
        if x == "tetris"{
            {NEW_TASKS
                .lock()
                .insert(0, VirtualAddress(tetris as usize));}
            unsafe { TASK_STARTED = true; }
            self.running_task = "tetris".to_string();
        } else if x == "htop" {
            ;
        } else if x == "help" {
            self.show_shell_manual();
            unsafe { TASK_STARTED = true; }
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
    /// The text is written quick and dirty to free areas on the screen.
    /// A more professional solution would be to write a method which parses a desired textfile
    /// (possibly containing control commands for formatting) and renders the text to a specified
    /// area on the screen.
    fn show_shell_manual(&mut self){
        write_at_background(
            "###### RTOS-SHELL - MANUAL ######",
            0,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "1. help   > Shows a full list of possible",
            2,
            35,
            Color::White,
            Color::Black
        );
        write_at_background(
            "            shell commands",
            3,
            35,
            Color::White,
            Color::Black
        );
        write_at_background(
            "2. tetris > Starts a funky tetris game",
            5,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "3. clock  > Adds a temporary clock to the",
            7,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "            left of the screen",
            8,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "4. reboot > Reboots the system",
            10,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "5. ctrl-c > Cancels the last command issued",
            12,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "            from the shell and activates new",
            13,
            35,
            Color::White,
            Color::Black,
        );
        write_at_background(
            "            input",
            14,
            35,
            Color::White,
            Color::Black
        );
    }

    fn parse_ctrl_command(&mut self, input: String) {
        if input == "c" {
            if self.running_task == "help" {
                self.reset_shell();
            } else {
                self.terminate_running_task = true;
            }
            self.input.clear();
            self.running_task = "".to_string();
        }
        self.parse_ctrl_command = false;
    }

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
