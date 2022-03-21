use crate::param::UART0;
use core::fmt::Error;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref UART: Mutex<Uart> = {
        let mut uart = Uart::new(UART0);
        uart.init();
        Mutex::new(uart)
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

pub struct Uart {
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

impl Uart {
    fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    fn init(&mut self) {
        let ptr = self.base_address as *mut u8;
        unsafe {}
    }

    fn put(&mut self, c: u8) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    fn get(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}
