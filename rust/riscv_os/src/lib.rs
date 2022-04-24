#![no_std]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]

extern crate alloc;

mod bio;
mod bmap;
mod console;
mod cpu;
mod fs;
mod kalloc;
mod kvm;
mod log;
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
mod superblock;
mod test;
mod trap;
mod uart;
mod virtio;
