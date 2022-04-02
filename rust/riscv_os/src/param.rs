// the kernel expects there to be RAM
// for use by the kernel and user pages
// from physical address 0x80000000 to PHYSTOP.
pub const KERNBASE: usize = 0x8000_0000;
pub const PHYSTOP: usize = KERNBASE + 128 * 1024 * 1024;
pub const PAGESIZE: usize = 4096;

pub const MAXVA: usize = 1 << (9 + 9 + 9 + 12 - 1);

// map the trampoline page to the highest address,
// in both user and kernel space.
// VirtAddr 0x3ffffff000
pub const TRAMPOLINE: usize = MAXVA - PAGESIZE;

// map kernel stacks beneath the trampoline,
// each surrounded by invalid guard pages.
pub const fn kstack(p: usize) -> usize {
    TRAMPOLINE - ((p + 1) * 5 * PAGESIZE)
}

// User memory layout.
// Address zero first:
//   text
//   original data and bss
//   fixed-size stack
//   expandable heap
//   ...
//   TRAPFRAME (p->trapframe, used by the trampoline)
//   TRAMPOLINE (the same page as in the kernel)
// VirtAddr 0x3fffffe000
pub const TRAPFRAME: usize = TRAMPOLINE - PAGESIZE;

pub const NCPU: usize = 8;
pub const NPROC: usize = 64;

// local interrupt controller, which contains the timer.
pub const CLINT: usize = 0x2000000;
pub const CLINT_MAP_SIZE: usize = 0x10000;

// qemu puts UART registers here in physical memory.
pub const UART0: usize = 0x1000_0000;
pub const UART0_MAP_SIZE: usize = PAGESIZE;
pub const UART0_IRQ: usize = 10;

// virtio mmio interface
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_MAP_SIZE: usize = PAGESIZE;
pub const VIRTIO0_IRQ: usize = 1;

// qemu puts programmable interrupt controller here.
pub const PLIC: usize = 0x0c000000;
pub const PLIC_MAP_SIZE: usize = 0x400000;
