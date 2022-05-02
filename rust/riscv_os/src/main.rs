#![no_std]
#![no_main]

#[allow(unused_imports)]
use riscv_os;
use riscv_os::{bootstrap, CPU_TABLE};

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
unsafe fn main() -> ! {
    bootstrap();
    CPU_TABLE.scheduler();
}
