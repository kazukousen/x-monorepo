#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(riscv_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::panic::PanicInfo;

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

#[test_case]
fn simple_allocation() {
    let v1 = Box::new(41);
    let v2 = Box::new(13);
    assert_eq!(*v1, 41);
    assert_eq!(*v2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1); // new
    for i in 0..1000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1); // new
}
