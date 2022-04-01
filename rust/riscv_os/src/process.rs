use crate::kvm::kvm_map;
use crate::page_table::{Page, PageTable, PteFlag, SinglePage};
use crate::param::{kstack, NPROC, PAGESIZE};
use crate::println;
use crate::proc::{Proc, ProcState, TrapFrame};
use crate::spinlock::SpinLock;
use array_macro::array;

pub struct ProcessTable {
    table: [Proc; NPROC],
    pid: SpinLock<usize>,
}

pub static mut PROCESS_TABLE: ProcessTable = ProcessTable::new();

impl ProcessTable {
    const fn new() -> Self {
        Self {
            table: array![_ => Proc::new(); NPROC],
            pid: SpinLock::new(0),
        }
    }

    // initialize the process table at boot time.
    // and allocate a page for each process's kernel stack.
    // map it high in memory, followed by an invalid guard page.
    pub unsafe fn proc_init(&mut self) {
        for (i, p) in self.table.iter_mut().enumerate() {
            let va = kstack(i);
            let pa = SinglePage::new_zeroed()
                .expect("process_table: insufficient memory for process's kernel stack");
            // map
            kvm_map(
                "process's kernel stack",
                va,
                pa as usize,
                PAGESIZE * 4,
                PteFlag::READ | PteFlag::WRITE,
            );
            // kstack
            p.data.get_mut().set_kstack(va);
        }
    }

    pub fn find_runnable(&mut self) -> Option<&mut Proc> {
        for p in self.table.iter_mut() {
            let mut locked = p.inner.lock();
            match locked.state {
                ProcState::Runnable => {
                    locked.state = ProcState::Allocated;
                    drop(locked);
                    return Some(p);
                }
                _ => {}
            }
            drop(locked)
        }

        None
    }

    fn alloc_pid(&self) -> usize {
        let ret: usize;
        let mut pid = self.pid.lock();
        ret = *pid;
        *pid += 1;
        ret
    }

    fn alloc_proc(&mut self) -> Option<&mut Proc> {
        let pid = self.alloc_pid();

        for p in self.table.iter_mut() {
            let mut locked = p.inner.lock();

            match locked.state {
                ProcState::Unused => {
                    // found an unused process

                    let pd = p.data.get_mut();

                    // hold trapframe pointer
                    pd.tf = unsafe { SinglePage::new_zeroed().ok()? as *mut TrapFrame };
                    // allocate trapframe page table
                    match PageTable::alloc_user_page_table(pd.tf as usize) {
                        Some(pgt) => {
                            pd.page_table = Some(pgt);
                        }
                        None => {
                            unsafe { SinglePage::drop(pd.tf as *mut u8) };
                            return None;
                        }
                    }
                    println!(
                        "alloc_proc: pid={} tf={:#x} satp={:#x}",
                        pid,
                        pd.tf as usize,
                        pd.page_table.as_ref().unwrap().as_satp()
                    );

                    pd.init_context();
                    locked.pid = pid;
                    locked.state = ProcState::Allocated;

                    println!("allocated pid: {}", locked.pid);

                    drop(locked);

                    return Some(p);
                }
                _ => drop(locked),
            }
        }

        None
    }

    pub fn user_init(&mut self) {
        let p = self.alloc_proc().expect("user_init: no free procs");

        // TODO: self.init_proc = p?

        p.user_init()
            .expect("user_init: failed process's initilization");

        let mut locked = p.inner.lock();
        locked.state = ProcState::Runnable;
    }
}
