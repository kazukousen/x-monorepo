use crate::kvm::kvm_map;
use crate::page_table::{Page, PageTable, PteFlag, SinglePage};
use crate::param::{kstack, NPROC, PAGESIZE};
use crate::proc::Proc;
use array_macro::array;

pub struct ProcessTable {
    table: [Proc; NPROC],
}

pub static mut PROCESS_TABLE: ProcessTable = ProcessTable::new();

impl ProcessTable {
    const fn new() -> Self {
        Self {
            table: array![i => Proc::new(i); NPROC],
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
}
