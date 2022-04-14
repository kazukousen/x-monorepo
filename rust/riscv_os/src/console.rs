use crate::{spinlock::SpinLock, uart};

pub fn putc(c: u8) {
    uart::putc_sync(c);
}

pub fn init() {
    uart::init();
}

const INPUT_BUF: usize = 128;

struct Console {
    buf: [u8; INPUT_BUF],
    r: usize, // read index
    w: usize, // write index
    e: usize, // edit index
}

impl Console {
    const fn new() -> Self {
        Self {
            buf: [0; INPUT_BUF],
            r: 0,
            w: 0,
            e: 0,
        }
    }
}

static CONSOLE: SpinLock<Console> = SpinLock::new(Console::new());

pub fn intr(c: u8) {
    let mut cons = CONSOLE.lock();
    match c {
        _ => {
            if c != 0 && cons.e - cons.r < INPUT_BUF {
                let c = if c == CTRL_CR { CTRL_LF } else { c };
                // echo back to the user
                putc(c);
                cons.e += 1;
                let i = cons.e % INPUT_BUF;
                cons.buf[i] = c;
            }
        }
    }
    drop(cons);
}

const CTRL_BS: u8 = 0x08;
const CTRL_LF: u8 = 0x0A;
const CTRL_CR: u8 = 0x0D;
