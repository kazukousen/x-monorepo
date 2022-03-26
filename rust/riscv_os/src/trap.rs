use crate::register::sstatus;

pub unsafe fn user_trap_ret() -> ! {
    sstatus::intr_off();

    loop{}
}

