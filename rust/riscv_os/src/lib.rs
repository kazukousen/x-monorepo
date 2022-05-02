#![no_std]
#![feature(alloc_error_handler)]
#![feature(new_uninit)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
use param::PAGESIZE;
use process::PROCESS_TABLE;
use virtio::DISK;
use core::{sync::atomic::{AtomicBool, Ordering}, ptr};

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    unsafe { ptr::write_volatile(QEMU_TEST0 as *mut u32, QEMU_EXIT_SUCCESS) };
}

pub const QEMU_TEST0: usize = 0x100000;
pub const QEMU_TEST0_MAP_SIZE: usize = PAGESIZE;
const QEMU_EXIT_SUCCESS: u32 = 0x5555;

#[cfg(test)]
#[no_mangle]
unsafe fn main() -> ! {
    test_main();
    loop {}
}

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
