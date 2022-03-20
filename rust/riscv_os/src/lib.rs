#![no_main]
#![no_std]

mod register;
mod rmain;
mod start;
mod uart;

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    loop {}
}
