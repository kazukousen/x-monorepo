use core::ptr;

use array_macro::array;

use crate::{
    param::NCPU,
    proc::{Proc, ProcState},
    process::PROCESS_TABLE,
    register::tp,
};

pub static mut CPU_TABLE: CpuTable = CpuTable::new();

pub struct CpuTable {
    table: [Cpu; NCPU],
}

impl CpuTable {
    #[inline]
    pub unsafe fn cpu_id() -> usize {
        tp::read()
    }

    const fn new() -> Self {
        Self {
            table: array![_ => Cpu::new(); NCPU],
        }
    }

    pub unsafe fn scheduler(&mut self) -> ! {
        let cpu = self.mycpu_mut();

        loop {
            if let Some(p) = PROCESS_TABLE.find_runnable() {
                cpu.proc = p as *mut _;

                let mut locked = p.inner.lock();
                locked.state = ProcState::Running;

                cpu.proc = ptr::null_mut();
                drop(locked);
            }
        }
    }

    unsafe fn mycpu_mut(&mut self) -> &mut Cpu {
        let id = Self::cpu_id();
        &mut self.table[id]
    }
}

struct Cpu {
    proc: *mut Proc,
}

impl Cpu {
    const fn new() -> Self {
        Self {
            proc: ptr::null_mut(),
        }
    }
}
