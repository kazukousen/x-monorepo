use core::ptr;

use array_macro::array;

use crate::{
    param::NCPU,
    proc::{Context, Proc, ProcInner, ProcState},
    process::PROCESS_TABLE,
    register::{sstatus, tp},
    spinlock::SpinLockGuard,
};

pub static mut CPU_TABLE: CpuTable = CpuTable::new();

pub struct CpuTable {
    table: [Cpu; NCPU],
}

impl CpuTable {
    #[inline]
    pub fn cpu_id() -> usize {
        unsafe { tp::read() }
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
                swtch(&mut cpu.scheduler as *mut _, ctx);

                cpu.proc = ptr::null_mut();
                drop(locked);
            }
        }
    }

    pub fn my_cpu_mut(&mut self) -> &mut Cpu {
        let id = Self::cpu_id();
        &mut self.table[id]
    }

    fn my_cpu(&self) -> &Cpu {
        let id = Self::cpu_id();
        &self.table[id]
    }

    pub fn my_proc(&mut self) -> &mut Proc {
        push_off();

        let p;

        let c = self.my_cpu();
        unsafe {
            p = &mut *c.proc;
        }

        pop_off();

        p
    }
}

pub struct Cpu {
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

    /// Switch to scheduler.
    /// Saves and restores intena because intena is a property of this
    /// kernel thread, not this CPU.
    /// Passing in and out a locked because we need to the lock during this function.
    pub fn sched<'a>(
        &mut self,
        locked: SpinLockGuard<'a, ProcInner>,
        ctx: *mut Context,
    ) -> SpinLockGuard<'a, ProcInner> {
        if self.noff != 1 {
            panic!("sched: multi locks");
        }
        if locked.state == ProcState::Running {
            panic!("sched: proc is running");
        }
        if sstatus::intr_get() {
            panic!("sched: interruptable");
        }

        let intena = self.intena;

        extern "C" {
            fn swtch(old: *mut Context, new: *mut Context);
        }
        unsafe {
            swtch(ctx, &mut self.scheduler as *mut _);
        }

        self.intena = intena;

        locked
    }

    pub unsafe fn yielding(&mut self) {
        if !self.proc.is_null() {
            let proc = self.proc.as_mut().unwrap();
            proc.yielding();
        }
    }
}

/// push_off/pop_off are like intr_off()/intr_on()
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
    let cpu = unsafe { CPU_TABLE.my_cpu_mut() };

    if sstatus::intr_get() {
        panic!(
            "pop_off: already interruputable noff={} intena={}",
            cpu.noff, cpu.intena
        );
    }

    if cpu.noff < 1 {
        panic!("pop_off");
    }
    cpu.noff -= 1;

    if cpu.noff == 0 && cpu.intena {
        sstatus::intr_on();
    }
}
