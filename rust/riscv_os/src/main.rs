#![no_main]
#![no_std]
#![feature(asm)]

mod uart;
mod register;

use core::panic::PanicInfo;

#[no_mangle]
fn start() -> ! {
    println!("Hello, World!");

    // 1. Perform some configurations that is only allowed in machine mode.
    // 2. Set a program counter to `main`.
    // 3. Disable paging by set `satp` to 0.
    // 4. Enable clock interrupts.
    // 5. Switch to supervisor mode and jump to `main`.

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {

    println!("{}", info);
    loop {}
}
