#![no_std]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]
#![feature(allocator_api)]

extern crate alloc;

mod bio;
mod console;
mod cpu;
mod fs;
mod kalloc;
mod kvm;
mod page_table;
mod param;
mod plic;
mod printf;
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

