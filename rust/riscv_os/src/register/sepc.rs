use core::arch::asm;

#[inline]
pub unsafe fn write(v: usize) {
    asm!("csrw sepc, {}", in(reg) v);
}
