
use crate::rmain;
use crate::register;
use core::arch::asm;

#[no_mangle]
unsafe fn start() -> ! {
    // 1. Perform some configurations that is only allowed in machine mode.
    register::mstatus::set_mpp(register::mstatus::MPPMode::Machine);

    // 2. Set a program counter to `main`.
    register::mepc::write(rmain::rust_main as usize);

    // 3. Disable paging by set `satp` to 0.
    register::satp::write(0);

    // 4. Delegate all interrupts and exceptions to supervisor mode.
    register::medeleg::write(0xffff);
    register::mideleg::write(0xffff);

    // 5. Enable clock interrupts.
    // TODO:
    // 6. Store each CPU's hart id in tp register, for cpuid().
    let id = register::mhartid::read();
    register::tp::write(id);

    // 7. Switch to supervisor mode and jump to `main`.
    asm!("mret");

    loop {}
}

