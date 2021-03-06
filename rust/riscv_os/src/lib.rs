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
pub mod printf;
mod proc;
mod process;
mod register;
mod sleeplock;
mod spinlock;
mod start;
mod superblock;
mod trap;
mod uart;
mod virtio;

use bio::BCACHE;
use core::panic::PanicInfo;
use core::{
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};
use cpu::CpuTable;
use cpu::CPU_TABLE;
use param::PAGESIZE;
use process::PROCESS_TABLE;
use virtio::DISK;

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    unsafe { ptr::write_volatile(QEMU_TEST0 as *mut u32, QEMU_EXIT_SUCCESS) };
}

// https://elixir.bootlin.com/qemu/v7.0.0/source/hw/riscv/virt.c#L73
pub const QEMU_TEST0: usize = 0x100000;
pub const QEMU_TEST0_MAP_SIZE: usize = PAGESIZE;
// https://elixir.bootlin.com/qemu/v7.0.0/source/include/hw/misc/sifive_test.h#L41
const QEMU_EXIT_SUCCESS: u32 = 0x5555;
const QEMU_EXIT_FAIL: u32 = 0x13333; // exit 1

pub static PANICKED: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
#[panic_handler]
pub fn panic(info: &PanicInfo<'_>) -> ! {
    test_panic_handler(info)
}

pub fn test_panic_handler(info: &PanicInfo<'_>) -> ! {
    println!("failed: {}", info);
    PANICKED.store(true, Ordering::Relaxed);
    unsafe { ptr::write_volatile(QEMU_TEST0 as *mut u32, QEMU_EXIT_FAIL) };
    loop {}
}

#[cfg(test)]
#[no_mangle]
unsafe fn main() -> ! {
    bootstrap();
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("\x1b[0;32m[ok]\x1b[0m");
    }
}

static STARTED: AtomicBool = AtomicBool::new(false);

pub unsafe fn bootstrap() -> ! {
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

    CPU_TABLE.scheduler();
}
