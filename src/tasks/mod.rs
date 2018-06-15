use vga_buffer;
use vga_buffer::Color;
use features::msleep;

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

pub fn idle_task() {
    early_trace!("IDLE");
    loop {
        unsafe {
            asm!("pause":::: "intel", "volatile");
        }
    }
}