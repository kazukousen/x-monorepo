use array_macro::array;

use crate::{register::tp, param::NCPU, process::PROCESS_TABLE, proc::ProcState};



pub fn cpu_id() -> usize {
    unsafe { tp::read() }
}

pub static mut CPU_TABLE: CpuTable = CpuTable::new();

pub struct CpuTable {
    table: [Cpu; NCPU],
}

impl CpuTable {
    const fn new() -> Self {
        Self {
            table: array![_ => Cpu::new(); NCPU],
        }
    }

    pub unsafe fn scheduler(&mut self) -> ! {

        let cpu_id = self.mycpu();

        loop {

            if let Some(p) = PROCESS_TABLE.find_runnable() {

                let mut locked = p.inner.lock();
                locked.state = ProcState::Running;
            }

        }
    }

    fn mycpu(&self) -> &Cpu {
        let id = cpu_id();
        &self.table[id]
    }
}

struct Cpu {
}

impl Cpu {
    const fn new() -> Self {
        Self {
        }
    }
}

