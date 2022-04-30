use core::cell::UnsafeCell;
use core::ptr;

use crate::cpu::{Cpu, CPU_TABLE};
use crate::file::File;
use crate::fs::{Inode, INODE_TABLE};
use crate::page_table::PageTable;
use crate::param::{NOFILE, PAGESIZE, ROOTDEV, ROOTIPATH};
use crate::spinlock::{SpinLock, SpinLockGuard};
use crate::{fs, println, trap};
use alloc::boxed::Box;
use array_macro::array;

mod elf;
mod syscall;

use self::syscall::Syscall;

#[repr(C)]
pub struct Context {
    pub ra: usize,
    pub sp: usize,

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

    fn clear(&mut self) {
        self.ra = 0;
        self.sp = 0;
        self.s1 = 0;
        self.s2 = 0;
        self.s3 = 0;
        self.s4 = 0;
        self.s5 = 0;
        self.s6 = 0;
        self.s7 = 0;
        self.s8 = 0;
        self.s9 = 0;
        self.s10 = 0;
        self.s11 = 0;
    }
}

#[repr(C)]
pub struct TrapFrame {
    /* 0 */ pub kernel_satp: usize,
    /* 8 */ pub kernel_sp: usize,
    /* 16 */ pub kernel_trap: usize,
    /* 24 */ pub epc: usize,
    /* 32 */ pub kernel_hartid: usize,
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
    pub kstack: usize,
    sz: usize,
    context: Context,
    name: [u8; 16],
    pub tf: *mut TrapFrame,
    pub page_table: Option<Box<PageTable>>,
    pub cwd: Option<Inode>,
    pub o_files: [Option<Box<File>>; NOFILE],
}

impl ProcessData {
    const fn new() -> Self {
        Self {
            kstack: 0,
            sz: 0,
            name: [0; 16],
            context: Context::new(),
            tf: ptr::null_mut(),
            page_table: None,
            cwd: None,
            o_files: array![_ => None; NOFILE],
        }
    }

    pub fn set_kstack(&mut self, kstack: usize) {
        self.kstack = kstack;
    }

    pub fn init_context(&mut self) {
        self.context.clear();
        self.context.ra = forkret as usize;
        self.context.sp = self.kstack + PAGESIZE * 4;
    }

    pub fn get_context(&mut self) -> *mut Context {
        &mut self.context as *mut _
    }
}

#[derive(PartialEq)]
pub enum ProcState {
    Unused,
    Runnable,
    Running,
    Allocated,
    Sleeping,
}

pub struct ProcInner {
    pub state: ProcState,
    pub pid: usize,
    // sleeping on channel
    pub chan: usize,
}

impl ProcInner {
    const fn new() -> Self {
        Self {
            state: ProcState::Unused,
            pid: 0,
            chan: 0,
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

        // allocate one user page and copy init's instructions
        // and data into it.
        pd.page_table.as_mut().unwrap().uvm_init(&INITCODE)?;
        pd.sz = PAGESIZE;

        // prepare for the very first "return" from kernel to user.
        let tf = unsafe { pd.tf.as_mut().unwrap() };
        tf.epc = 0; // user program counter
        tf.sp = PAGESIZE; // user stack poiner

        let init_name = b"initcode\0";
        unsafe {
            ptr::copy_nonoverlapping(init_name.as_ptr(), pd.name.as_mut_ptr(), init_name.len());
        }
        pd.cwd = Some(
            INODE_TABLE
                .namei(&ROOTIPATH)
                .expect("cannot find root inode by b'/'"),
        );

        Ok(())
    }

    pub unsafe fn syscall(&mut self) {
        let pd = self.data.get_mut();
        let tf = pd.tf.as_mut().unwrap();

        let num = tf.a7;

        let ret = match num {
            1 => pd.sys_fork(),
            7 => pd.sys_exec(),
            10 => pd.sys_dup(),
            15 => pd.sys_open(),
            16 => pd.sys_write(),
            _ => {
                panic!("unknown syscall: {}", num);
            }
        };

        tf.a0 = match ret {
            Ok(ret) => ret,
            Err(msg) => {
                println!("syscall error: {}", msg);
                -1isize as usize
            }
        };
    }

    pub unsafe fn yielding(&self) {
        let mut locked = self.inner.lock();
        if locked.state == ProcState::Running {
            let ctx = &mut (*self.data.get()).context;
            locked.state = ProcState::Runnable;
            locked = CPU_TABLE.my_cpu_mut().sched(locked, ctx);
        }
        drop(locked);
    }

    /// Atomically release lock and sleep on chan.
    /// The passed-in guard must not be the proc's guard to avoid deadlock.
    pub fn sleep<'a, T>(&self, chan: usize, lk: SpinLockGuard<'a, T>) -> SpinLockGuard<'a, T> {
        let mut locked = self.inner.lock();

        // Go to sleep
        locked.chan = chan;
        locked.state = ProcState::Sleeping;

        // unlock lk
        let weaked = lk.weak();

        unsafe {
            let cpu = CPU_TABLE.my_cpu_mut();
            locked = cpu.sched(locked, &mut (*self.data.get()).context);
        }

        // Tidy up.
        locked.chan = 0;
        weaked.lock()
    }
}

pub fn either_copy(is_user: bool, src: *const u8, dst: *mut u8, count: usize) {
    if is_user {
        // TODO:
        panic!("either_copy_out: not implemented");
    } else {
        unsafe { ptr::copy(src, dst, count) };
    }
}

static mut FIRST: bool = true;

pub unsafe fn forkret() -> ! {
    CPU_TABLE.my_proc().inner.unlock();

    if FIRST {
        FIRST = false;
        fs::init(ROOTDEV);
    }

    trap::user_trap_ret();
}

/// first user program that calls exec("/init")
static INITCODE: [u8; 51] = [
    0x17, 0x05, 0x00, 0x00, 0x13, 0x05, 0x05, 0x02, 0x97, 0x05, 0x00, 0x00, 0x93, 0x85, 0x05, 0x02,
    0x9d, 0x48, 0x73, 0x00, 0x00, 0x00, 0x89, 0x48, 0x73, 0x00, 0x00, 0x00, 0xef, 0xf0, 0xbf, 0xff,
    0x2f, 0x69, 0x6e, 0x69, 0x74, 0x00, 0x00, 0x01, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00,
];
