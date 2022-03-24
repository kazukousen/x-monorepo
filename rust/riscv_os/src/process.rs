use crate::kvm::kvm_map;
use crate::page_table::{Page, PageTable, PteFlag, SinglePage};
use crate::param::{kstack, NPROC, PAGESIZE};
use crate::proc::{Proc, TrapFrame};
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
            table: array![i => Proc::new(i); NPROC],
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
            kvm_map(va, pa as usize, PAGESIZE, PteFlag::Read | PteFlag::Write);
            p.data.get_mut().set_kstack(va);
        }
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
            // TODO excl.lock
            let pd = p.data.get_mut();
            pd.tf = unsafe { SinglePage::new_zeroed().ok()? as *mut TrapFrame };
            match PageTable::alloc_user_page_table(pd.tf as usize) {
                Some(pgt) => {
                    pd.page_table = Some(pgt);
                },
                None => {
                    unsafe { SinglePage::drop(pd.tf as *mut u8) };
                    return None;
                },
            }
            // TODO: drop lock
        }

        None
    }
}

