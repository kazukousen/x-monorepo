use crate::println;

use super::Proc;


type SysResult = Result<usize, ()>;

pub trait Syscall {
    fn sys_exec(&mut self) -> SysResult;
}

impl Syscall for Proc {
    fn sys_exec(&mut self) -> SysResult {

        println!("sys_exec: TODO");

        Err(())
    }
}
