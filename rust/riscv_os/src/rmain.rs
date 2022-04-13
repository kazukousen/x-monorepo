use crate::bio::BCACHE;
use crate::console;
use crate::cpu::CpuTable;
use crate::cpu::CPU_TABLE;
use crate::kalloc;
use crate::kvm;
use crate::plic;
use crate::println;
use crate::process::PROCESS_TABLE;
use crate::trap;
use crate::virtio::DISK;
use core::sync::atomic::{AtomicBool, Ordering};

static STARTED: AtomicBool = AtomicBool::new(false);

/// start() jumps here in supervisor mode on all CPUs.
#[no_mangle]
pub unsafe fn main() -> ! {
    let cpu_id = CpuTable::cpu_id();
    if cpu_id == 0 {
        console::init();
        println!("xv6 kernel in Rust is booting...");
        kalloc::heap_init(); // physical memory allocator
        kvm::init(); // create the kernel page table
        kvm::init_hart(); // turn on paging
        PROCESS_TABLE.proc_init(); // process table
        trap::init_hart(); // install kernel trap vector
        plic::init(); // set up interrupt controller
        plic::init_hart(cpu_id); // ask PLIC for device interrupts
        BCACHE.init(); // buffer cache
        DISK.lock().init(); // emulated hard disk
        PROCESS_TABLE.user_init(); // first user process

        STARTED.store(true, Ordering::SeqCst);
    } else {
        while !STARTED.load(Ordering::SeqCst) {}
        println!("hart {} starting...", cpu_id);
        kvm::init_hart(); // turn on paging
        trap::init_hart(); // install kernel trap handler
        plic::init_hart(cpu_id); // ask PLIC for device interrupts
    }

    CPU_TABLE.scheduler();
}
