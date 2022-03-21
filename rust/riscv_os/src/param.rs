pub const KERNBASE: usize = 0x8000_0000;
pub const PHYSTOP: usize = KERNBASE + 128 * 1024 * 1024;
pub const PAGESIZE: usize = 4096;

// local interrupt controller, which contains the timer.
pub const CLINT: usize = 0x2000000;
pub const CLINT_MAP_SIZE: usize = 0x10000;

// qemu puts UART registers here in physical memory.
pub const UART0: usize = 0x1000_0000;
pub const UART0_MAP_SIZE: usize = PAGESIZE;

// virtio mmio interface
pub const VIRTIO0: usize = 0x10001000;
pub const VIRTIO0_MAP_SIZE: usize = PAGESIZE;

// qemu puts programmable interrupt controller here.
pub const PLIC: usize = 0x0c000000;
pub const PLIC_MAP_SIZE: usize = 0x400000;
