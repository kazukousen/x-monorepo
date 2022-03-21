#![no_main]
#![no_std]

mod kalloc;
mod param;
mod process;
mod register;
mod rmain;
mod start;
mod uart;
mod vm;

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    loop {}
}
