pub const KERNBASE: usize = 0x8000_0000;
pub const PHYSTOP: usize = KERNBASE + 128 * 1024 * 1024;
pub const PAGESIZE: usize = 4096;
