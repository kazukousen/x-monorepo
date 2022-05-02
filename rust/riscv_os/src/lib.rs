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
mod file;
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
mod sleeplock;
mod spinlock;
mod start;
mod superblock;
mod test;
mod trap;
mod uart;
mod virtio;

use bio::BCACHE;
use cpu::CpuTable;
pub use cpu::CPU_TABLE;
use process::PROCESS_TABLE;
use virtio::DISK;
use core::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

pub unsafe fn bootstrap() {
    let cpu_id = CpuTable::cpu_id();
    if cpu_id == 0 {
        console::init();
        println!("xv6 kernel in Rust is booting...");
        kalloc::heap_init(); // physical memory allocator
        kvm::init(); // create the kernel page table
        kvm::init_hart(); // turn on paging
        PROCESS_TABLE.proc_init(); // process table
        trap::init_hart(); // install kernel trap vector
        plic::init(); // set up interrupt controller
        plic::init_hart(cpu_id); // ask PLIC for device interrupts
        BCACHE.init(); // buffer cache
        DISK.lock().init(); // emulated hard disk
        PROCESS_TABLE.user_init(); // first user process

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {}
        println!("hart {} starting...", cpu_id);
        kvm::init_hart(); // turn on paging
        trap::init_hart(); // install kernel trap handler
        plic::init_hart(cpu_id); // ask PLIC for device interrupts
    }
}
