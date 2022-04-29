use core::{mem, ptr, str};

use alloc::boxed::Box;
use array_macro::array;

use crate::{log::LOG, param::NOFILE, println};

use super::{elf, ProcessData};

type SysResult = Result<usize, &'static str>;

pub trait Syscall {
    fn sys_fork(&mut self) -> SysResult; // 1
    fn sys_exec(&mut self) -> SysResult; // 7
    fn sys_open(&mut self) -> SysResult; // 10
    fn sys_dup(&mut self) -> SysResult; // 15
    fn sys_write(&mut self) -> SysResult; // 16
}

pub const MAXARG: usize = 16;
pub const MAXARGLEN: usize = 64;

impl Syscall for ProcessData {
    fn sys_fork(&mut self) -> SysResult {
        panic!("sys_fork: unimplemented");
    }

    fn sys_exec(&mut self) -> SysResult {
        let mut path: [u8; 128] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let nul_pos = self.arg_str(0, &mut path)?;
        let path_str = unsafe { str::from_utf8_unchecked(&path[0..nul_pos]) };

        if path_str == "/init" {
            // crate::test::run_tests();
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

        elf::load(self, &path, &argv)
    }

    fn sys_open(&mut self) -> SysResult {
        let mut path: [u8; 128] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let nul_pos = self.arg_str(0, &mut path)?;
        let path_str = unsafe { str::from_utf8_unchecked(&path[0..nul_pos]) };
        let omode = self.arg_i32(1)?;

        println!("sys_open: path={} omode={}", path_str, omode);

        Ok(0)
    }

    fn sys_dup(&mut self) -> SysResult {
        Ok(0)
    }

    fn sys_write(&mut self) -> SysResult {
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

    #[inline]
    fn arg_i32(&self, n: usize) -> Result<i32, &'static str> {
        let addr = self.arg_raw(n)?;
        Ok(addr as i32)
    }

    fn arg_fd(&self, n: usize) -> Result<(), &'static str> {
        let fd = self.arg_i32(n)?;
        if fd < 0 {
            return Err("file descriptor must be greater than or equal to 0");
        }
        if fd >= NOFILE.try_into().unwrap() {
            return Err("file descriptor must be less than NOFILE");
        }

        Ok(())
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
