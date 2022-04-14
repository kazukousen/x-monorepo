use crate::{cpu, param::UART0, printf::PANICKED};
use core::{ptr, sync::atomic::Ordering};

const RHR: usize = 0;
const THR: usize = 0;
const IER: usize = 1;
const FCR: usize = 2;
const ISR: usize = 2;
const LCR: usize = 3;
const LSR: usize = 5;

pub fn init() {
    unsafe {
        ptr::write_volatile((UART0 + IER) as *mut u8, 0x00);
        ptr::write_volatile((UART0 + LCR) as *mut u8, 0x80);
        ptr::write_volatile((UART0 + 0) as *mut u8, 0x03);
        ptr::write_volatile((UART0 + 1) as *mut u8, 0x00);
        ptr::write_volatile((UART0 + LCR) as *mut u8, 0x03);
        ptr::write_volatile((UART0 + FCR) as *mut u8, 0x07);
        ptr::write_volatile((UART0 + IER) as *mut u8, 0x03);
    }
}

// alternate version of putc() that doesn't
// use interrupts, for use by kernel printf() and
// to echo characters. it spins waiting for the uart's
// output register to be empty.
pub fn putc_sync(c: u8) {
    cpu::push_off();

    if PANICKED.load(Ordering::Relaxed) {
        loop {}
    }

    unsafe {
        while ptr::read_volatile((UART0 + LSR) as *const u8) & (1 << 5) == 0 {}
        ptr::write_volatile((UART0 + THR) as *mut u8, c);
    }
    cpu::pop_off();
}
