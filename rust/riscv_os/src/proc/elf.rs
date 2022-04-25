#[repr(C)]
pub struct ELFHeader {
    pub magic: u32,
    elf: [u8; 12],
    typed: u16,
    machine: u16,
    version: u32,
    entry: usize,
    phoff: usize,
    shoff: usize,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

impl ELFHeader {
    pub fn empty() -> Self {
        Self {
            magic: 0,
            elf: [0; 12],
            typed: 0,
            machine: 0,
            version: 0,
            entry: 0,
            phoff: 0,
            shoff: 0,
            flags: 0,
            ehsize: 0,
            phentsize: 0,
            phnum: 0,
            shentsize: 0,
            shnum: 0,
            shstrndx: 0,
        }
    }
}

pub const MAGIC: u32 = 0x464C457F;
