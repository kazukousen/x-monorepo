use core::ptr;

use array_macro::array;

use crate::{
    param::NCPU,
    proc::{Proc, ProcState, Context},
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
        let cpu = self.my_cpu_mut();

        loop {
            if let Some(p) = PROCESS_TABLE.find_runnable() {
                cpu.proc = p as *mut _;

                let mut locked = p.inner.lock();
                locked.state = ProcState::Running;

                extern "C" {
                    fn swtch(old: *mut Context, new: *mut Context);
                }

                swtch(&mut cpu.context as *mut _, p.data.get_mut().get_context());

                cpu.proc = ptr::null_mut();
                drop(locked);
            }
        }
    }

    unsafe fn my_cpu_mut(&mut self) -> &mut Cpu {
        let id = Self::cpu_id();
        &mut self.table[id]
    }

    unsafe fn my_cpu(&self) -> &Cpu {
        let id = Self::cpu_id();
        &self.table[id]
    }

    pub fn my_proc(&mut self) -> &mut Proc {
        let p;

        unsafe {
            let c = self.my_cpu();

            p = &mut *c.proc;
        }

        p
    }
}

struct Cpu {
    proc: *mut Proc,
    context: Context,
}

impl Cpu {
    const fn new() -> Self {
        Self {
            proc: ptr::null_mut(),
            context: Context::new(),
        }
    }
}
