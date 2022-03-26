use core::ptr;

use array_macro::array;

use crate::{
    param::NCPU,
    println,
    proc::{Context, Proc, ProcState},
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
            table: array![i => Cpu::new(i); NCPU],
        }
    }

    pub unsafe fn scheduler(&mut self) -> ! {
        let cpu = self.my_cpu_mut();

        loop {
            if let Some(p) = PROCESS_TABLE.find_runnable() {
                println!("find a process. cpu={}", cpu.hartid);

                cpu.proc = p as *mut _;

                let mut locked = p.inner.lock();
                locked.state = ProcState::Running;

                extern "C" {
                    fn swtch(old: *mut Context, new: *mut Context);
                }

                swtch(&mut cpu.scheduler as *mut _, p.data.get_mut().get_context());

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
    hartid: usize,
    proc: *mut Proc,
    scheduler: Context,
}

impl Cpu {
    const fn new(hartid: usize) -> Self {
        Self {
            hartid,
            proc: ptr::null_mut(),
            scheduler: Context::new(),
        }
    }
}
