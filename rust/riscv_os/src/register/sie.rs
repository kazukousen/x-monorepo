use core::arch::asm;

#[inline]
unsafe fn read() -> usize {
    let ret: usize;
    asm!("csrr {}, sie", out(reg) ret);
    ret
}

#[inline]
unsafe fn write(v: usize) {
    asm!("csrw sie, {}", in(reg) v);
}

pub unsafe fn enable_supervisor_all() {
    let mut sie = read();
    sie |= 1 << 1; // Software Interrupt
    sie |= 1 << 5; // Timer Interrupt
    sie |= 1 << 9; // External Interrupt
    write(sie);
}
