use core::mem;

use crate::{
    cpu::{self, CpuTable, CPU_TABLE},
    param, plic,
    register::{self, scause::ScauseType},
    spinlock::SpinLock,
    uart,
    virtio::DISK,
};

/// set up to take exceptions and traps while in the kernel.
pub unsafe fn init_hart() {
    extern "C" {
        fn kernelvec();
    }
    register::stvec::write(kernelvec as usize);
}

#[no_mangle]
pub unsafe fn kerneltrap() {
    let sepc = register::sepc::read();
    let sstatus = register::sstatus::read();

    if !register::sstatus::is_from_supervisor() {
        panic!("kerneltrap: not from supervisor mode");
    }

    if register::sstatus::intr_get() {
        panic!("kerneltrap: interrupts enabled");
    }

    handle_trap(false);

    // the yield() may have caused some traps to occur,
    // so restore trap registers for use by kernelvec.S's sepc instruction.
    register::sepc::write(sepc);
    register::sstatus::write(sstatus);
}

unsafe fn handle_trap(is_user: bool) {
    let scause = register::scause::get_type();
    match scause {
        ScauseType::IntSExt => {
            // this is a supervisor external interrupt, via PLIC.

            let irq = plic::claim();

            match irq as usize {
                param::VIRTIO0_IRQ => {
                    DISK.lock().intr();
                }
                param::UART0_IRQ => {
                    uart::intr();
                }
                _ => {}
            }

            if irq > 0 {
                plic::complete(irq);
            }
        }
        ScauseType::IntSSoft => {
            // println!("kerneltrap: handling timer interrupt");

            if cpu::CpuTable::cpu_id() == 0 {
                clock_intr();
            }

            register::sip::clear_ssip();

            CPU_TABLE.my_cpu_mut().yielding();
        }
        ScauseType::ExcEcall => {
            if !is_user {
                panic!("kerneltrap: handling syscall");
            }
            // an interrupt will change sstatus &c registers,
            // so don't enable until done with those registers.
            register::sstatus::intr_on();

            cpu::CPU_TABLE.my_proc().syscall();
        }
        ScauseType::Unknown(v) => {
            panic!("handle_trap: scause {:#x}", v);
        }
    }
}

static TICKS: SpinLock<usize> = SpinLock::new(0);

fn clock_intr() {
    let mut locked = TICKS.lock();
    *locked += 1;
    drop(locked)
}

/// return to user space
pub unsafe fn user_trap_ret() -> ! {
    let p = cpu::CPU_TABLE.my_proc();

    register::sstatus::intr_off();

    extern "C" {
        fn uservec();
        fn trampoline();
    }

    // send syscalls, interrupts, and exceptions to trampoline.S
    register::stvec::write(param::TRAMPOLINE + (uservec as usize - trampoline as usize));

    let pd = p.data.get_mut();

    let tf = pd.tf.as_mut().unwrap();

    tf.kernel_satp = register::satp::read();
    tf.kernel_sp = pd.kstack + param::PAGESIZE * 4;
    tf.kernel_trap = user_trap as usize;
    tf.kernel_hartid = CpuTable::cpu_id();

    // set S Previous Privilege mode to User.
    register::sstatus::prepare_user_ret();

    // set S Exception Program Counter to the saved user pc.
    register::sepc::write(tf.epc);

    // tell trampoline.S the user page table to switch to.
    let satp = pd.page_table.as_ref().unwrap().as_satp();

    // jump to trampoline.S at the top of memory, which
    // switches to the user page table, restores user registers,
    // and switches to user mode with sret.
    extern "C" {
        fn userret();
    }
    let user_ret_virt = param::TRAMPOLINE + (userret as usize - trampoline as usize);
    let user_ret_virt: extern "C" fn(usize, usize) -> ! = mem::transmute(user_ret_virt);

    user_ret_virt(param::TRAPFRAME, satp);
}

#[no_mangle]
unsafe extern "C" fn user_trap() {
    if register::sstatus::is_from_supervisor() {
        panic!("usertrap: not from user mode");
    }

    // send interrupts and exceptions to kerneltrap(),
    // since we're now in the kernel.
    extern "C" {
        fn kernelvec();
    }
    register::stvec::write(kernelvec as usize);

    let p = cpu::CPU_TABLE.my_proc();
    let tf = p.data.get_mut().tf.as_mut().unwrap();
    // save user program counter.
    tf.epc = register::sepc::read();

    handle_trap(true);

    user_trap_ret();
}
