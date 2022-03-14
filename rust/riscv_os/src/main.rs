#![no_main]
#![no_std]

mod uart;

use core::panic::PanicInfo;

#[no_mangle]
fn start() -> ! {
    println!("Hello, World!");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {

    println!("{}", info);
    loop {}
}
