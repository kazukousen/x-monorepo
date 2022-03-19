use crate::println;

/// start() jumps here in supervisor mode on all CPUs.
pub fn rust_main() -> ! {
    println!("Hello, World!");
    loop {}
}
