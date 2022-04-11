#![no_std]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]
#![feature(allocator_api)]

extern crate alloc;

use core::panic::PanicInfo;

mod bio;
mod cpu;
mod fs;
mod kalloc;
mod kvm;
mod page_table;
mod param;
mod plic;
mod proc;
mod process;
mod register;
mod rmain;
mod sleeplock;
mod spinlock;
mod start;
mod trap;
mod uart;
mod virtio;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("panic: {}", info);
    loop {}
}

#[no_mangle]
fn abort() -> ! {
    panic!("abort");
}
