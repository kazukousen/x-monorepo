use core::ptr;

use array_macro::array;

use crate::{
    param::NCPU,
    println,
    proc::{Context, Proc, ProcState},
    process::PROCESS_TABLE,
    register::{sstatus, tp},
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
            // Avoid deadlock by ensuring that devices can interrupt.
            sstatus::intr_on();

            if let Some(p) = PROCESS_TABLE.find_runnable() {
                cpu.proc = p as *mut _;

                let mut locked = p.inner.lock();
                locked.state = ProcState::Running;

                let ctx = p.data.get_mut().get_context();
                {
                    let ctx = ctx.as_mut().unwrap();
                    println!(
                        "scheduler: new context ra={:#x} stack={:#x}",
                        ctx.ra, ctx.sp
                    );
                }

                swtch(&mut cpu.scheduler as *mut _, ctx);

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
        push_off();

        let p;

        unsafe {
            let c = self.my_cpu();

            p = &mut *c.proc;
        }

        pop_off();

        p
    }
}

struct Cpu {
    hartid: usize,
    proc: *mut Proc,
    scheduler: Context,
    // Depth of push_off() nesting.
    noff: u8,
    // Were interruputs enabled before push_off()?
    intena: bool,
}

impl Cpu {
    const fn new(hartid: usize) -> Self {
        Self {
            hartid,
            proc: ptr::null_mut(),
            scheduler: Context::new(),
            noff: 0,
            intena: false,
        }
    }
}

pub fn push_off() {
    let old = sstatus::intr_get();
    sstatus::intr_off();

    let cpu = unsafe { CPU_TABLE.my_cpu_mut() };

    if cpu.noff == 0 {
        cpu.intena = old;
    }
    cpu.noff += 1;
}

pub fn pop_off() {
    if sstatus::intr_get() {
        panic!("pop_off: interruputable");
    }

    let cpu = unsafe { CPU_TABLE.my_cpu_mut() };

    if cpu.noff < 1 {
        panic!("pop_off");
    }
    cpu.noff -= 1;

    if cpu.noff == 0 && cpu.intena {
        sstatus::intr_on();
    }
}
