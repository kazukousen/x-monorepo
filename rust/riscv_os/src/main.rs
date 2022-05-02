#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(riscv_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use riscv_os::{bootstrap, CPU_TABLE};

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
unsafe fn main() -> ! {
    bootstrap();
    #[cfg(test)]
    test_main();
    CPU_TABLE.scheduler();
}
