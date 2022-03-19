#![no_main]
#![no_std]

mod register;
mod uart;
mod rmain;

use core::panic::PanicInfo;

#[no_mangle]
unsafe fn start() -> ! {
    // 1. Perform some configurations that is only allowed in machine mode.
    register::mstatus::set_mpp(register::mstatus::MPPMode::Machine);
    // 2. Set a program counter to `main`.
    register::mepc::write(rmain::rust_main as usize);
    // 3. Disable paging by set `satp` to 0.
    register::satp::write(0);
    // 4. Delegate all interrupts and exceptions to supervisor mode.
    // 5. Enable clock interrupts.
    // 6. Store each CPU's hart id in tp register, for cpuid().
    // 7. Switch to supervisor mode and jump to `main`.

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    loop {}
}
