//! This was a very first implementation of a clock. It was mainly used to learn and work with some
//! rust specific stuff like match and structs.
//! the code is marked as `dead_code` and will not be used in the os.
use vga_buffer;
use vga_buffer::Color;

#[allow(dead_code)]
pub struct Clock {
    row: u8,
    col: u8,
}

impl Clock {
    #[allow(dead_code)]
    pub fn new(row: u8, col: u8) -> Self {
        let color = Color::LightGreen;
        vga_buffer::write_at(":", row, col + 5, color);
        vga_buffer::write_at(":", row, col + 2, color);
        vga_buffer::write_at("0", row, col, color);
        vga_buffer::write_at("0", row, col + 1, color);
        vga_buffer::write_at("0", row, col + 3, color);
        vga_buffer::write_at("0", row, col + 4, color);
        vga_buffer::write_at("0", row, col + 6, color);
        vga_buffer::write_at("0", row, col + 7, color);
        Clock { row, col }
    }

    #[allow(dead_code)]
    pub fn uptime(&self) {
        let color = Color::LightGreen;
        loop {
            match vga_buffer::read_at(self.row as usize, (self.col + 7) as usize) {
                48 => vga_buffer::write_at("1", self.row, self.col + 7, color),
                49 => vga_buffer::write_at("2", self.row, self.col + 7, color),
                50 => vga_buffer::write_at("3", self.row, self.col + 7, color),
                51 => vga_buffer::write_at("4", self.row, self.col + 7, color),
                52 => vga_buffer::write_at("5", self.row, self.col + 7, color),
                53 => vga_buffer::write_at("6", self.row, self.col + 7, color),
                54 => vga_buffer::write_at("7", self.row, self.col + 7, color),
                55 => vga_buffer::write_at("8", self.row, self.col + 7, color),
                56 => vga_buffer::write_at("9", self.row, self.col + 7, color),
                57 => {
                    vga_buffer::write_at("0", self.row, self.col + 7, color);
                    match vga_buffer::read_at(self.row as usize, (self.col + 6) as usize) {
                        48 => vga_buffer::write_at("1", self.row, self.col + 6, color),
                        49 => vga_buffer::write_at("2", self.row, self.col + 6, color),
                        50 => vga_buffer::write_at("3", self.row, self.col + 6, color),
                        51 => vga_buffer::write_at("4", self.row, self.col + 6, color),
                        52 => vga_buffer::write_at("5", self.row, self.col + 6, color),
                        53 => {
                            vga_buffer::write_at("0", self.row, self.col + 6, color);
                            self.increase_minute();
                        }
                        _ => vga_buffer::write_at("0", self.row, self.col + 6, color),
                    }
                }
                _ => vga_buffer::write_at("0", self.row, self.col + 7, color),
            }
            self.sleep();
        }
    }

    #[allow(dead_code)]
    fn increase_minute(&self) {
        let color = Color::LightGreen;
        match vga_buffer::read_at(self.row as usize, (self.col + 4) as usize) {
            48 => vga_buffer::write_at("1", self.row, self.col + 4, color),
            49 => vga_buffer::write_at("2", self.row, self.col + 4, color),
            50 => vga_buffer::write_at("3", self.row, self.col + 4, color),
            51 => vga_buffer::write_at("4", self.row, self.col + 4, color),
            52 => vga_buffer::write_at("5", self.row, self.col + 4, color),
            53 => vga_buffer::write_at("6", self.row, self.col + 4, color),
            54 => vga_buffer::write_at("7", self.row, self.col + 4, color),
            55 => vga_buffer::write_at("8", self.row, self.col + 4, color),
            56 => vga_buffer::write_at("9", self.row, self.col + 4, color),
            57 => {
                vga_buffer::write_at("0", self.row, self.col + 4, color);
                match vga_buffer::read_at(self.row as usize, (self.col + 3) as usize) {
                    48 => vga_buffer::write_at("1", self.row, self.col + 3, color),
                    49 => vga_buffer::write_at("2", self.row, self.col + 3, color),
                    50 => vga_buffer::write_at("3", self.row, self.col + 3, color),
                    51 => vga_buffer::write_at("4", self.row, self.col + 3, color),
                    52 => vga_buffer::write_at("5", self.row, self.col + 3, color),
                    53 => {
                        vga_buffer::write_at("0", self.row, self.col + 3, color);
                        self.increase_hour();
                    }
                    _ => vga_buffer::write_at("0", self.row, self.col + 3, color),
                }
            }
            _ => vga_buffer::write_at("0", self.row, self.col + 4, color),
        }
    }

    #[allow(dead_code)]
    fn increase_hour(&self) {
        let color = Color::LightGreen;
        match (
            vga_buffer::read_at(self.row as usize, self.col as usize),
            vga_buffer::read_at(self.row as usize, (self.col + 1) as usize),
        ) {
            (48, 48) => vga_buffer::write_at("1", self.row, self.col + 1, color),
            (48, 49) => vga_buffer::write_at("2", self.row, self.col + 1, color),
            (48, 50) => vga_buffer::write_at("3", self.row, self.col + 1, color),
            (48, 51) => vga_buffer::write_at("4", self.row, self.col + 1, color),
            (48, 52) => vga_buffer::write_at("5", self.row, self.col + 1, color),
            (48, 53) => vga_buffer::write_at("6", self.row, self.col + 1, color),
            (48, 54) => vga_buffer::write_at("7", self.row, self.col + 1, color),
            (48, 55) => vga_buffer::write_at("8", self.row, self.col + 1, color),
            (48, 56) => vga_buffer::write_at("9", self.row, self.col + 1, color),
            (48, 57) => {
                vga_buffer::write_at("0", self.row, self.col + 1, color);
                vga_buffer::write_at("1", self.row, self.col, color);
            }
            (49, 48) => vga_buffer::write_at("1", self.row, self.col + 1, color),
            (49, 49) => vga_buffer::write_at("2", self.row, self.col + 1, color),
            (49, 50) => vga_buffer::write_at("3", self.row, self.col + 1, color),
            (49, 51) => vga_buffer::write_at("4", self.row, self.col + 1, color),
            (49, 52) => vga_buffer::write_at("5", self.row, self.col + 1, color),
            (49, 53) => vga_buffer::write_at("6", self.row, self.col + 1, color),
            (49, 54) => vga_buffer::write_at("7", self.row, self.col + 1, color),
            (49, 55) => vga_buffer::write_at("8", self.row, self.col + 1, color),
            (49, 56) => vga_buffer::write_at("9", self.row, self.col + 1, color),
            (49, 57) => {
                vga_buffer::write_at("0", self.row, self.col + 1, color);
                vga_buffer::write_at("2", self.row, self.col, color);
            }
            (50, 51) => {
                vga_buffer::write_at("0", self.row, self.col + 1, color);
                vga_buffer::write_at("0", self.row, self.col, color);
            }
            (50, 48) => vga_buffer::write_at("1", self.row, self.col + 1, color),
            (50, 49) => vga_buffer::write_at("2", self.row, self.col + 1, color),
            (50, 50) => vga_buffer::write_at("3", self.row, self.col + 1, color),
            _ => vga_buffer::write_at("0", self.row, self.col + 1, color),
        }
    }

    #[allow(dead_code)]
    pub fn sleep(&self) {
        //1342302694
        for i in 0..500_000 {
            let _x = i;
        }
    }
}
