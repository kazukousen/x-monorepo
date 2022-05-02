#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(riscv_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::{panic::PanicInfo, sync::atomic::Ordering};

use riscv_os::{bootstrap, println, CPU_TABLE, PANICKED};

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
unsafe fn main() -> ! {
    #[cfg(test)]
    test_main();
    bootstrap();
    CPU_TABLE.scheduler();
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("panic: {}", info);
    PANICKED.store(true, Ordering::Relaxed);
    loop {}
}
