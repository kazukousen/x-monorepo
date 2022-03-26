use core::ptr;

use array_macro::array;

use crate::{
    param::NCPU,
    println,
    proc::{Context, Proc, ProcState},
    process::PROCESS_TABLE,
    register::{tp, sie, sstatus},
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
        extern "C" {
            fn swtch(old: *mut Context, new: *mut Context);
        }
        let cpu = self.my_cpu_mut();

        loop {
            // ensure devices can interrupt
            intr_on();

            if let Some(p) = PROCESS_TABLE.find_runnable() {
                cpu.proc = p as *mut _;

                let mut locked = p.inner.lock();
                locked.state = ProcState::Running;

                let ctx_ptr = p.data.get_mut().get_context();

                {
                    let ctx = ctx_ptr.as_mut().unwrap();
                    println!(
                        "scheduler: new context ra={:#x} stack={:#x}",
                        ctx.ra, ctx.sp
                    );
                }

                swtch(&mut cpu.scheduler as *mut _, ctx_ptr);

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

#[inline]
unsafe fn intr_on() {
    sie::intr_on();
    sstatus::intr_on();
}

