use core::arch::asm;

#[inline]
unsafe fn read() -> usize {
    let ret: usize;
    asm!("csrr {}, scause", out(reg) ret);
    ret
}

const INTERRUPT: usize = 0x8000000000000000;
const INTERRUPT_SUPERVISOR_SOFTWARE: usize = INTERRUPT + 1;
const INTERRUPT_SUPERVISOR_EXTERNAL: usize = INTERRUPT + 9;

pub enum ScauseType {
    Unknown(usize),
    IntSSoft,
    IntSExt,
}

#[inline]
pub unsafe fn get_type() -> ScauseType {
    let scause = read();
    match scause {
        INTERRUPT_SUPERVISOR_SOFTWARE => ScauseType::IntSSoft,
        INTERRUPT_SUPERVISOR_EXTERNAL => ScauseType::IntSExt,
        v => ScauseType::Unknown(v - INTERRUPT),
    }
}
