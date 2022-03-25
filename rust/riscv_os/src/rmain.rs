use crate::cpu::CpuTable;
use crate::cpu::CPU_TABLE;
use crate::kalloc;
use crate::kvm;
use crate::println;
use crate::process::PROCESS_TABLE;
use core::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
pub unsafe fn main() -> ! {
    let cpu_id = CpuTable::cpu_id();
    if cpu_id == 0 {
        println!("Hello, World! in Rust {}", cpu_id);

        // initialize physical memory allocator
        kalloc::heap_init();
        // initialize the kernel page table
        kvm::init();
        kvm::init_hart();
        // initialize the process table and allocate a page for each process's kernel stack.
        PROCESS_TABLE.proc_init();
        PROCESS_TABLE.user_init();

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {}
        println!("hart {} starting", cpu_id);
        kvm::init_hart();
    }

    CPU_TABLE.scheduler();
}
