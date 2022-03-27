use core::mem;

use crate::{
    cpu, param,
    register::{self, sstatus},
};

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
    tf.kernel_sp = pd.kstack + param::PAGESIZE;
    tf.kernel_trap = user_trap as usize;
    tf.kernel_hartid = register::tp::read();

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
