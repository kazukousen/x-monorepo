use core::{mem, str};

use crate::println;

use super::ProcessData;

type SysResult = Result<usize, ()>;

pub trait Syscall {
    fn sys_exec(&mut self) -> SysResult;
}

impl Syscall for ProcessData {
    fn sys_exec(&mut self) -> SysResult {
        let mut path: [u8; 128] = unsafe { mem::MaybeUninit::uninit().assume_init() };
        match self.arg_str(0, &mut path) {
            Ok(_) => match str::from_utf8(&path) {
                Ok(s) => {
                    println!("sys_exec: debug {}", s);
                }
                Err(e) => {
                    println!("sys_exec: invalid path: {}", e);
                }
            },
            Err(msg) => {
                println!("sys_exec: {}", msg);
            }
        }

        Err(())
    }
}

impl ProcessData {
    #[inline]
    fn arg_str(&self, n: usize, dst: &mut [u8]) -> Result<(), &'static str> {
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
