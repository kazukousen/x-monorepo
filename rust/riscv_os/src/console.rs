use crate::uart;


pub fn putc(c: u8) {
    uart::putc_sync(c);
}

pub fn init() {
    uart::init();
}

