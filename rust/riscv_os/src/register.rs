
pub mod mstatus {
    use core::arch::asm;
    unsafe fn read() -> usize {
        let ret: usize;
        asm!("csrr $0, mstatus":"=r"(ret):::"volatile");
        ret
    }
}