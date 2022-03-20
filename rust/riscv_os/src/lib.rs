#![no_main]
#![no_std]

mod register;
mod uart;
mod rmain;
mod start;

use core::panic::PanicInfo;
use core::arch::asm;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    loop {}
}
