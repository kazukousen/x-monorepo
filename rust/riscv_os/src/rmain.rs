use crate::kalloc;
use crate::println;
use crate::process;
use core::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
pub fn main() -> ! {
    let cpu_id = process::cpu_id();
    if cpu_id == 0 {
        println!("Hello, World! in Rust {}", cpu_id);

        kalloc::kinit();

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {}
        println!("hart {} starting", cpu_id);
    }
    loop {}
}
