use vga_buffer::*;
use alloc::{Vec, string::ToString};
use alloc::string::String;
use tasks::{NEW_TASKS, tetris, TASK_STARTED};
use x86_64::VirtualAddress;
use trace::*;

pub struct Shell {
    separating_line_color: Color,
    background_color: Color,
    default_cursor_position: (u8, u8),
    current_cursor_position: (u8, u8),
    input: String,
    input_history: Vec<String>,
    pub terminate_running_task: bool,
    parse_ctrl_command: bool,
    pub running_task: usize,
    active_screen: (u8, u8, u8, u8),
}

impl Shell {
    #[allow(dead_code)]
    pub fn new(separating_line_color: Color, background_color: Color, current_cursor_position: (u8, u8)) -> Self {
        Shell {
            separating_line_color,
            background_color,
            default_cursor_position: current_cursor_position,
            current_cursor_position: current_cursor_position,
            input: "".to_string(),
            input_history: Vec::new(),
            terminate_running_task: false,
            parse_ctrl_command: false,
            running_task: 0,
            active_screen: (40,80,0,20)
        }
    }

    pub fn init_shell(&mut self){
        self.print_separating_line();
        let cursor_position_height = self.current_cursor_position.0;
        self.print_prompt(cursor_position_height, 0);
    }

    pub fn clear_active_screen(&mut self){
        for col in self.active_screen.0..self.active_screen.1 {
            for row in self.active_screen.2..self.active_screen.3 {
                write_at_background(" ", row, col, Color::Black, Color::Black);
            }
        }
    }

    pub fn print_separating_line(&mut self){
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

    pub fn print_input(&mut self, input: String) {
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
            //delete blinking cursor in current line
            write_at_background(
                " ",
                self.current_cursor_position.0,
                self.current_cursor_position.1,
                Color::Black,
                Color::Black,
            );
            self.parse_command();
        } else if input == "CTRL_LEFT" || input == "CTRL_RIGHT" {
            self.parse_ctrl_command = true;
        } else if self.parse_ctrl_command == true{
            self.parse_ctrl_command(input);
        } else {
            self.print_input(input);
        }
    }

    fn parse_command(&mut self){
        let x = self.input.to_string();
        self.input_history.push(x.clone());
        if x == "tetris"{
            {NEW_TASKS
                .lock()
                .insert(0, VirtualAddress(tetris as usize));}
            unsafe {TASK_STARTED = true;}
        } else if x == "htop" {
            //{NEW_TASKS.lock().insert(0, VirtualAddress(htop as usize));}
            unsafe {TASK_STARTED = true;}
        } else if x == "" {
            ;
        } else {
            if self.current_cursor_position.0 as usize >= BUFFER_HEIGHT - 1{
                write_at_background(
                    "unknown command                                                           ",
                    BUFFER_HEIGHT as u8 - 1,
                    0,
                    Color::Red,
                    Color::Black,
                );
                self.input_history.push("unknown command".to_string());
                self.print_shift_history();
            } else {
                self.current_cursor_position.0 += 1;
                write_at_background(
                    "unknown command                                                           ",
                    self.current_cursor_position.0,
                    0,
                    Color::Red,
                    Color::Black,
                );
                self.input_history.push("unknown command".to_string());
                self.current_cursor_position.0 += 1;
                self.current_cursor_position.1 = self.default_cursor_position.1;
                let cursor_position_height = self.current_cursor_position.0;
                self.print_prompt(cursor_position_height, 0);
            }

        }
        self.input.clear();
    }

    fn parse_ctrl_command(&mut self, input: String){
        if input == "c" {
            self.terminate_running_task = true;
            trace_fatal!("self.terminate_running_task == {:?}", self.terminate_running_task);
            self.input.clear();
        }
        self.parse_ctrl_command = false;
    }

    pub fn reset_shell(&mut self){
        self.terminate_running_task = false;
        self.clear_active_screen();
        if self.current_cursor_position.0 as usize >= BUFFER_HEIGHT - 1{
            self.print_shift_history();
        } else {
            self.current_cursor_position.0 += 1;
        }
        self.current_cursor_position.1 = self.default_cursor_position.1;
        let cursor_position_height = self.current_cursor_position.0;
        self.print_prompt(cursor_position_height, 0);
        unsafe {TASK_STARTED = false;}
    }

    fn print_shift_history(&mut self){
        let mut cnt: usize = 1;
        for row in self.default_cursor_position.0 as usize..BUFFER_HEIGHT-1 {
            let mut history_entry = &self.input_history[(self.input_history.len()-1) - (cnt - 1)].to_string();
            if history_entry == "unknown command" {
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
        /// clear last line and print a prompt
        self.print_prompt(BUFFER_HEIGHT as u8 - 1, 0);
        write_at_background(
            "                                                                    ",
            BUFFER_HEIGHT as u8 - 1,
            self.default_cursor_position.1,
            Color::Black,
            Color::Black,
        );
    }
}