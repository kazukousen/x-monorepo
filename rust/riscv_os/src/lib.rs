#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]
#![feature(allocator_api)]

#[macro_use]
extern crate alloc;

use core::panic::PanicInfo;

mod kalloc;
mod kvm;
mod page_table;
mod param;
mod proc;
mod process;
mod register;
mod rmain;
mod spinlock;
mod start;
mod uart;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{}", info);
    loop {}
}
