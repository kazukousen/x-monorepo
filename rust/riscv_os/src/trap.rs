use core::mem;

use crate::{
    cpu::{self, CpuTable},
    param,
    register::{self, sstatus}, println,
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
    if register::sstatus::is_spp() {
        panic!("");
    }
}

/// return to user space
pub unsafe fn user_trap_ret() -> ! {

    let p = cpu::CPU_TABLE.my_proc();

    sstatus::intr_off();

    extern "C" {
        fn uservec();
        fn trampoline();
    }

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

    user_ret_virt(param::TRAMPOLINE, satp);
}

fn user_trap() {}
