use core::{mem, str};

use crate::println;

use super::ProcessData;

type SysResult = Result<usize, &'static str>;

pub trait Syscall {
    fn sys_exec(&mut self) -> SysResult;
}

impl Syscall for ProcessData {
    fn sys_exec(&mut self) -> SysResult {
        let mut path: [u8; 128] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        let nul_pos = self.arg_str(0, &mut path)?;
        let path = unsafe { str::from_utf8_unchecked(&path[0..nul_pos]) };

        if path == "/init" {
            println!("sys_exec: {}", path);
            crate::test::run_tests();
        }

        Ok(0)
    }
}

impl ProcessData {
    #[inline]
    fn arg_str(&self, n: usize, dst: &mut [u8]) -> Result<usize, &'static str> {
        let addr = self.arg_raw(n);
        self.page_table.as_ref().unwrap().copy_in_str(dst, addr)
    }

    fn arg_raw(&self, n: usize) -> usize {
        let tf = unsafe { self.tf.as_ref().unwrap() };
        match n {
            0 => tf.a0,
            1 => tf.a1,
            2 => tf.a2,
            3 => tf.a3,
            4 => tf.a4,
            5 => tf.a5,
            _ => panic!("arg raw"),
        }
    }
}
