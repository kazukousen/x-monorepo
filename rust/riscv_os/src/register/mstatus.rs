use core::arch::asm;

#[inline]
unsafe fn read() -> usize {
    let ret: usize;
    asm!("csrr {}, mstatus", out(reg) ret);
    ret
}

#[inline]
unsafe fn write(v: usize) {
    asm!("csrw mstatus, {}", in(reg) v);
}

/// Machine Previous Privilege Mode
pub enum MPPMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

pub unsafe fn set_mpp(mode: MPPMode) {
    let mut mstatus = read();
    mstatus &= !(3 << 11);
    mstatus |= match mode {
        MPPMode::User => 0 << 11,
        MPPMode::Supervisor => 1 << 11,
        MPPMode::Machine => 3 << 11,
    };
    write(mstatus);
}

pub unsafe fn enable_interrupt(mode: MPPMode) {
    let mut mstatus = read();
    mstatus |= match mode {
        MPPMode::User => 1 << 0,
        MPPMode::Supervisor => 1 << 1,
        MPPMode::Machine => 1 << 3,
    };
    write(mstatus);
}

pub unsafe fn disable_interrupt(mode: MPPMode) {
    let mut mstatus = read();
    mstatus &= match mode {
        MPPMode::User => !(1 << 0),
        MPPMode::Supervisor => !(1 << 1),
        MPPMode::Machine => !(1 << 3),
    };
    write(mstatus);
}
