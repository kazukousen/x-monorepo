use core::cell::UnsafeCell;
use core::ptr;

use crate::page_table::PageTable;
use crate::param::PAGESIZE;
use crate::println;
use crate::spinlock::SpinLock;
use alloc::boxed::Box;

#[repr(C)]
pub struct Context {
    ra: usize,
    sp: usize,

    // callee saved
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }
}

#[repr(C)]
pub struct TrapFrame {
    /* 0 */ kernel_satp: usize,
    /* 8 */ kernel_sp: usize,
    /* 16 */ kernel_trap: usize,
    /* 24 */ epc: usize,
    /* 32 */ kernel_hartid: usize,
    /* 40 */ ra: usize,
    /* 48 */ sp: usize,
    /* 56 */ gp: usize,
    /* 64 */ tp: usize,
    /*  72 */ t0: usize,
    /*  80 */ t1: usize,
    /*  88 */ t2: usize,
    /*  96 */ s0: usize,
    /* 104 */ s1: usize,
    /* 112 */ a0: usize,
    /* 120 */ a1: usize,
    /* 128 */ a2: usize,
    /* 136 */ a3: usize,
    /* 144 */ a4: usize,
    /* 152 */ a5: usize,
    /* 160 */ a6: usize,
    /* 168 */ a7: usize,
    /* 176 */ s2: usize,
    /* 184 */ s3: usize,
    /* 192 */ s4: usize,
    /* 200 */ s5: usize,
    /* 208 */ s6: usize,
    /* 216 */ s7: usize,
    /* 224 */ s8: usize,
    /* 232 */ s9: usize,
    /* 240 */ s10: usize,
    /* 248 */ s11: usize,
    /* 256 */ t3: usize,
    /* 264 */ t4: usize,
    /* 272 */ t5: usize,
    /* 280 */ t6: usize,
}

pub struct ProcessData {
    kstack: usize,
    sz: usize,
    context: Context,
    pub tf: *mut TrapFrame,
    pub page_table: Option<Box<PageTable>>,
}

impl ProcessData {
    const fn new() -> Self {
        Self {
            kstack: 0,
            sz: 0,
            context: Context::new(),
            tf: ptr::null_mut(),
            page_table: None,
        }
    }

    pub fn set_kstack(&mut self, kstack: usize) {
        self.kstack = kstack;
    }

    pub fn init_context(&mut self) {
        extern "Rust" {
            fn forkret();
        }

        self.context.ra = forkret as usize;
        self.context.sp = self.kstack + PAGESIZE;
    }

    pub fn get_context(&mut self) -> *mut Context {
        &mut self.context as *mut _
    }
}

pub enum ProcState {
    Unused,
    Runnable,
    Running,
    Allocated,
}

pub struct ProcInner {
    pub state: ProcState,
    pub pid: usize,
}

impl ProcInner {
    const fn new() -> Self {
        Self {
            state: ProcState::Unused,
            pid: 0,
        }
    }
}

pub struct Proc {
    pub inner: SpinLock<ProcInner>,
    pub data: UnsafeCell<ProcessData>,
}

impl Proc {
    pub const fn new() -> Self {
        Self {
            inner: SpinLock::new(ProcInner::new()),
            data: UnsafeCell::new(ProcessData::new()),
        }
    }

    pub fn user_init(&mut self) -> Result<(), &'static str> {
        let pd = self.data.get_mut();
        pd.page_table.as_mut().unwrap().uvm_init()?;

        Ok(())
    }
}

#[no_mangle]
fn forkret() {
    println!("forkret");
}
