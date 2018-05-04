use vga_buffer::Color;
use vga_buffer;

pub struct Clock{}

impl Clock {

    pub fn new() -> Self {
        let color = Color::LightGreen;
        vga_buffer::write_at(":", 0, 76, color);
        vga_buffer::write_at(":", 0, 73, color);
        vga_buffer::write_at("0", 0, 71, color);
        vga_buffer::write_at("0", 0, 72, color);
        vga_buffer::write_at("0", 0, 74, color);
        vga_buffer::write_at("0", 0, 75, color);
        vga_buffer::write_at("0", 0, 77, color);
        vga_buffer::write_at("0", 0, 78, color);
        Clock{}
    }

    pub fn uptime(&self) {
        let color = Color::LightGreen;
        loop {
            match vga_buffer::read_at(0, 78) {
                48 => vga_buffer::write_at("1", 0, 78, color),
                49 => vga_buffer::write_at("2", 0, 78, color),
                50 => vga_buffer::write_at("3", 0, 78, color),
                51 => vga_buffer::write_at("4", 0, 78, color),
                52 => vga_buffer::write_at("5", 0, 78, color),
                53 => vga_buffer::write_at("6", 0, 78, color),
                54 => vga_buffer::write_at("7", 0, 78, color),
                55 => vga_buffer::write_at("8", 0, 78, color),
                56 => vga_buffer::write_at("9", 0, 78, color),
                57 => {
                    vga_buffer::write_at("0", 0, 78, color);
                    match vga_buffer::read_at(0, 77 as usize) {
                        48 => vga_buffer::write_at("1", 0, 77, color),
                        49 => vga_buffer::write_at("2", 0, 77, color),
                        50 => vga_buffer::write_at("3", 0, 77, color),
                        51 => vga_buffer::write_at("4", 0, 77, color),
                        52 => vga_buffer::write_at("5", 0, 77, color),
                        53 => {
                            vga_buffer::write_at("0", 0, 77, color);
                            self.increase_minute();
                        }
                        _ => vga_buffer::write_at("X", 0, 77, color),
                    }
                }
                _ => vga_buffer::write_at("X", 0, 78, color),
            }
            self.sleep();
        }
    }

    fn increase_minute(&self) {
        let color = Color::LightGreen;
        match vga_buffer::read_at(0, 75) {
            48 => vga_buffer::write_at("1", 0, 75, color),
            49 => vga_buffer::write_at("2", 0, 75, color),
            50 => vga_buffer::write_at("3", 0, 75, color),
            51 => vga_buffer::write_at("4", 0, 75, color),
            52 => vga_buffer::write_at("5", 0, 75, color),
            53 => vga_buffer::write_at("6", 0, 75, color),
            54 => vga_buffer::write_at("7", 0, 75, color),
            55 => vga_buffer::write_at("8", 0, 75, color),
            56 => vga_buffer::write_at("9", 0, 75, color),
            57 => {
                vga_buffer::write_at("0", 0, 75, color);
                match vga_buffer::read_at(0, 74 as usize) {
                    48 => vga_buffer::write_at("1", 0, 74, color),
                    49 => vga_buffer::write_at("2", 0, 74, color),
                    50 => vga_buffer::write_at("3", 0, 74, color),
                    51 => vga_buffer::write_at("4", 0, 74, color),
                    52 => vga_buffer::write_at("5", 0, 74, color),
                    53 => {
                        vga_buffer::write_at("0", 0, 74, color);
                        self.increase_hour();
                    }
                    _ => vga_buffer::write_at("X", 0, 74, color),
                }
            }
            _ => vga_buffer::write_at("X", 0, 75, color),
        }
    }

    fn increase_hour(&self) {
        let color = Color::LightGreen;
        match (vga_buffer::read_at(0, 71), vga_buffer::read_at(0, 72)) {
            (48, 48) => vga_buffer::write_at("1", 0, 72, color),
            (48, 49) => vga_buffer::write_at("2", 0, 72, color),
            (48, 50) => vga_buffer::write_at("3", 0, 72, color),
            (48, 51) => vga_buffer::write_at("4", 0, 72, color),
            (48, 52) => vga_buffer::write_at("5", 0, 72, color),
            (48, 53) => vga_buffer::write_at("6", 0, 72, color),
            (48, 54) => vga_buffer::write_at("7", 0, 72, color),
            (48, 55) => vga_buffer::write_at("8", 0, 72, color),
            (48, 56) => vga_buffer::write_at("9", 0, 72, color),
            (48, 57) => {
                vga_buffer::write_at("0", 0, 72, color);
                vga_buffer::write_at("1", 0, 71, color);
            }
            (49, 48) => vga_buffer::write_at("1", 0, 72, color),
            (49, 49) => vga_buffer::write_at("2", 0, 72, color),
            (49, 50) => vga_buffer::write_at("3", 0, 72, color),
            (49, 51) => vga_buffer::write_at("4", 0, 72, color),
            (49, 52) => vga_buffer::write_at("5", 0, 72, color),
            (49, 53) => vga_buffer::write_at("6", 0, 72, color),
            (49, 54) => vga_buffer::write_at("7", 0, 72, color),
            (49, 55) => vga_buffer::write_at("8", 0, 72, color),
            (49, 56) => vga_buffer::write_at("9", 0, 72, color),
            (49, 57) => {
                vga_buffer::write_at("0", 0, 72, color);
                vga_buffer::write_at("2", 0, 71, color);
            }
            (50, 51) => {
                vga_buffer::write_at("0", 0, 72, color);
                vga_buffer::write_at("0", 0, 71, color);
            }
            (50, 48) => vga_buffer::write_at("1", 0, 72, color),
            (50, 49) => vga_buffer::write_at("2", 0, 72, color),
            (50, 50) => vga_buffer::write_at("3", 0, 72, color),
            _ => vga_buffer::write_at("X", 0, 72, color),
        }
    }

    pub fn sleep(&self) {
        //1342302694
        for i in 0..500_000 {
            let _x = i;
        }
    }
}
