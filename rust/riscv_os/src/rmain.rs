use crate::kalloc;
use crate::kvm;
use crate::println;
use crate::process;
use core::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
pub unsafe fn main() -> ! {
    let cpu_id = process::cpu_id();
    if cpu_id == 0 {
        println!("Hello, World! in Rust {}", cpu_id);

        // initialize physical memory allocator
        kalloc::heap_init();
        kvm::init();
        kvm::init_hart();

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {}
        println!("hart {} starting", cpu_id);
        kvm::init_hart();
    }
    loop {}
}
