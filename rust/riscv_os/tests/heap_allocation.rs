#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(riscv_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use core::panic::PanicInfo;

#[test_case]
fn simple_allocation() {
    let v1 = Box::new(41);
    let v2 = Box::new(13);
    assert_eq!(*v1, 41);
    assert_eq!(*v2, 13);
}

#[cfg(test)]
#[no_mangle]
unsafe fn main() {
    riscv_os::bootstrap();
    test_main();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    riscv_os::test_panic_handler(info)
}

