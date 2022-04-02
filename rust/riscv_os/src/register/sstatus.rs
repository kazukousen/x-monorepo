use core::arch::asm;

const SIE: usize = 1 << 1;
const SPIE: usize = 1 << 5;
const SPP: usize = 1 << 8;

#[inline]
pub unsafe fn read() -> usize {
    let ret: usize;
    asm!("csrr {}, sstatus", out(reg) ret);
    ret
}

#[inline]
pub unsafe fn write(v: usize) {
    asm!("csrw sstatus, {}", in(reg) v);
}

#[inline]
pub fn intr_on() {
    unsafe {
        write(read() | SIE);
    }
}

#[inline]
pub fn intr_off() {
    unsafe {
        write(read() & !SIE);
    }
}

// are device interrupts enabled?
#[inline]
pub fn intr_get() -> bool {
    let x = unsafe { read() };
    (x & SIE) != 0
}

#[inline]
pub fn is_from_supervisor() -> bool {
    unsafe { read() & SPP != 0 }
}

#[inline]
pub fn prepare_user_ret() {
    unsafe {
        let mut x = read();
        x &= !SPP;
        x |= SPIE;
        write(x);
    }
}
