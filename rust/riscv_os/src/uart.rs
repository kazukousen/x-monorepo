use crate::cpu;
use crate::param::UART0;
use crate::spinlock::SpinLock;
use core::fmt::Error;
use core::fmt::Write;
use lazy_static::lazy_static;

lazy_static! {
    static ref UART: SpinLock<Uart> = {
        let mut uart = Uart::new(UART0);
        uart.init();
        SpinLock::new(uart)
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    UART.lock().write_fmt(args).unwrap();
}

struct Uart {
    base_address: usize,
}

impl Write for Uart {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        for c in out.bytes() {
            self.put(c);
        }
        Ok(())
    }
}

const RHR: usize = 0;
const THR: usize = 0;
const IER: usize = 1;
const FCR: usize = 2;
const ISR: usize = 2;
const LCR: usize = 3;
const LSR: usize = 5;

impl Uart {
    fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    fn init(&mut self) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            ptr.add(IER).write_volatile(0x00);
            ptr.add(LCR).write_volatile(0x80);
            ptr.add(0).write_volatile(0x03);
            ptr.add(1).write_volatile(0x00);
            ptr.add(LCR).write_volatile(0x03);
            ptr.add(FCR).write_volatile(0x07);
            ptr.add(IER).write_volatile(0x03);
        }
    }

    fn put(&mut self, c: u8) {
        cpu::push_off();
        let ptr = self.base_address as *mut u8;
        unsafe {
            while !(ptr.add(LSR).read_volatile() & (1 << 5) > 0) {}
            ptr.add(THR).write_volatile(c);
        }
        cpu::pop_off();
    }

    fn get(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(LSR).read_volatile() & 1 == 0 {
                None
            } else {
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

