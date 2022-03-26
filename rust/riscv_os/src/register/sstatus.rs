use core::arch::asm;

const SIE: usize = 1 << 1;
const SPIE: usize = 1 << 5;
const SPP: usize = 1 << 8;

#[inline]
unsafe fn read() -> usize {
    let ret: usize;
    asm!("csrr {}, sstatus", out(reg) ret);
    ret
}

#[inline]
unsafe fn write(v: usize) {
    asm!("csrw sstatus, {}", in(reg) v);
}

#[inline]
pub fn intr_on() {
    unsafe { write(read() | SIE); }
}

#[inline]
pub fn intr_off() {
    unsafe { write(read() & !SIE); }
}

#[inline]
pub fn intr_get() -> bool {
    let x = unsafe { read() };
    (x & SIE) != 0
}
