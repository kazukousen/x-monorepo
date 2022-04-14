use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{console, spinlock::SpinLock};

struct Print;

impl fmt::Write for Print {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            console::putc(c);
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    static PRINT: SpinLock<()> = SpinLock::new(());

    if PANICKED.load(Ordering::Relaxed) {
        Print.write_fmt(args).expect("printf: error");
        return
    }
    let locked = PRINT.lock();
    Print.write_fmt(args).expect("printf: error");
    drop(locked);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::printf::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub static PANICKED: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    crate::println!("panic: {}", info);
    PANICKED.store(true, Ordering::Relaxed);
    loop {}
}

#[no_mangle]
fn abort() -> ! {
    panic!("abort");
}
