use crate::println;

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
pub fn main() -> ! {
    println!("Hello, World! in Rust");
    loop {}
}
