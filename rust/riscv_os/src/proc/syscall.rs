use core::{mem, str};

use alloc::boxed::Box;
use array_macro::array;

use crate::println;

use super::ProcessData;

type SysResult = Result<usize, &'static str>;

pub trait Syscall {
    fn sys_exec(&mut self) -> SysResult;
}

const MAXARG: usize = 16;
const MAXARGLEN: usize = 64;

impl Syscall for ProcessData {
    fn sys_exec(&mut self) -> SysResult {
        let mut path: [u8; 128] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let nul_pos = self.arg_str(0, &mut path)?;
        let path = unsafe { str::from_utf8_unchecked(&path[0..nul_pos]) };

        if path == "/init" {
            crate::test::run_tests();
        }

        let arg_base_addr = self.arg_raw(1)?;
        let mut argv: [Option<Box<[u8; MAXARGLEN]>>; MAXARG] = array![_ => None; MAXARG];
        for i in 0..MAXARG {
            let uarg = self.fetch_addr(arg_base_addr + i * mem::size_of::<usize>())?;
            if uarg == 0 {
                break;
            }

            match Box::<[u8; MAXARGLEN]>::try_new_zeroed() {
                Ok(b) => unsafe { argv[i] = Some(b.assume_init()) },
                Err(_) => {
                    return Err("sys_exec: cannot allocate kernel space to copy arg");
                }
            }

            // copy arg to kernel space
            self.fetch_str(uarg, argv[i].as_deref_mut().unwrap())?;
        }
        unsafe {
            println!(
                "sys_exec: {} {}",
                path,
                str::from_utf8_unchecked(argv[0].as_deref().unwrap())
            );
        }

        Ok(0)
    }
}

impl ProcessData {
    #[inline]
    fn arg_str(&self, n: usize, dst: &mut [u8]) -> Result<usize, &'static str> {
        let addr = self.arg_raw(n)?;
        self.fetch_str(addr, dst)
    }

    #[inline]
    fn fetch_str(&self, addr: usize, dst: &mut [u8]) -> Result<usize, &'static str> {
        self.page_table.as_ref().unwrap().copy_in_str(dst, addr)
    }

    #[inline]
    fn arg_raw(&self, n: usize) -> Result<usize, &'static str> {
        let tf = unsafe { self.tf.as_ref().unwrap() };
        match n {
            0 => Ok(tf.a0),
            1 => Ok(tf.a1),
            2 => Ok(tf.a2),
            3 => Ok(tf.a3),
            4 => Ok(tf.a4),
            5 => Ok(tf.a5),
            _ => Err("arg raw"),
        }
    }

    fn fetch_addr(&self, addr: usize) -> Result<usize, &'static str> {
        if addr >= self.sz || addr + mem::size_of::<usize>() > self.sz {
            return Err("fetch_addr size");
        }
        let mut dst: usize = 0;
        self.page_table.as_ref().unwrap().copy_in(
            &mut dst as *mut usize as *mut u8,
            addr,
            mem::size_of::<usize>(),
        )?;
        Ok(dst)
    }
}
