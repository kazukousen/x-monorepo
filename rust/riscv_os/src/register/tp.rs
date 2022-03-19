use core::arch::asm;

#[inline]
pub unsafe fn write(v: usize) {
    asm!("mv tp, {}", in(reg) v);
}
