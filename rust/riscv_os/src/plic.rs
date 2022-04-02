use core::ptr;

use crate::param;

#[inline]
unsafe fn write(offset: usize, v: u32) {
    let dst = (param::PLIC + offset) as *mut u32;
    ptr::write_volatile(dst, v);
}

pub unsafe fn init() {
    write(param::UART0_IRQ * 4, 1);
    write(param::VIRTIO0_IRQ * 4, 1);
}

pub unsafe fn init_hart(hart: usize) {
    write(SENABLE + SENABLE_HART * hart, (1 << param::UART0_IRQ) | (1 << param::VIRTIO0_IRQ));
    write(SPRIORITY + SPRIORITY_HART * hart, 0);
}

const SENABLE: usize = 0x2080;
const SENABLE_HART: usize = 0x100;
const SPRIORITY: usize = 0x201000;
const SPRIORITY_HART: usize = 0x2000;
