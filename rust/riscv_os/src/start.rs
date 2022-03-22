use crate::register;
use core::arch::asm;
use crate::param::NCPU;

#[no_mangle]
static STACK0: [u8; 4096 * NCPU] = [0; 4096 * NCPU];

#[no_mangle]
static TIMER_SCRATCH: [[usize; 5]; NCPU] = [[0; 5]; NCPU];

#[no_mangle]
unsafe fn start() -> ! {
    // 1. Perform some configurations that is only allowed in machine mode.
    register::mstatus::set_mpp(register::mstatus::MPPMode::Machine);

    extern "Rust" {
        fn main();
    }

    // 2. Set a program counter to `main`.
    register::mepc::write(main as usize);

    // 3. Disable paging by set `satp` to 0.
    register::satp::write(0);

    // 4. Delegate all interrupts and exceptions to supervisor mode.
    register::medeleg::write(0xffff);
    register::mideleg::write(0xffff);

    // 5. Enable interrupt in supervisor mode
    register::sie::enable_supervisor_all();

    // 5. Enable clock interrupts.
    timerinit();

    // 6. Store each CPU's hart id in tp register, for cpuid().
    let id = register::mhartid::read();
    register::tp::write(id);

    // 7. Switch to supervisor mode and jump to `main`.
    asm!("mret");

    loop {}
}

unsafe fn timerinit() {
    let id = register::mhartid::read();

    // ask the CLINT for a timer interrupt.
    let interval = 1000000; // cycles; about 1/10th second in qemu.
    register::clint::add_mtimecmp(id, interval);

    let mut arr = TIMER_SCRATCH[id];
    arr[3] = register::clint::CLINT_MTIMECMP + 8 * id;
    arr[4] = interval as usize;
    register::mscratch::write(arr.as_ptr() as u64);

    // Set the machine-mode trap handler.
    extern "C" {
        fn timervec();
    }
    register::mtvec::write(timervec as usize);

    // Enable machine interrupt.
    register::mstatus::enable_interrupt(register::mstatus::MPPMode::Machine);

    // Enable machine-mode timer interrupt.
    register::mie::enable_machine_timer_interrupt();
}
