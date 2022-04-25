use core::mem;

use crate::{fs::INODE_TABLE, log::LOG, page_table::PageTable, println, proc::ProcessData};

#[repr(C)]
pub struct ELFHeader {
    magic: u32,
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
    fn empty() -> Self {
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

const MAGIC: u32 = 0x464C457F;

pub fn load(p: &ProcessData, path: &[u8]) -> Result<(), &'static str> {
    LOG.begin_op();

    let inode = match INODE_TABLE.namei(&path) {
        None => {
            LOG.end_op();
            return Err("sys_exec: cannot find inode by given path");
        }
        Some(inode) => inode,
    };

    let mut idata = inode.ilock();

    println!("sys_exec: size={}", idata.size());

    let mut elfhdr = ELFHeader::empty();
    let elfhdr_ptr = &mut elfhdr as *mut ELFHeader as *mut u8;

    idata
        .readi(false, elfhdr_ptr, 0, mem::size_of::<ELFHeader>())
        .or(Err("cannot read from inode"))?;

    if elfhdr.magic != MAGIC {
        drop(idata);
        drop(inode);
        LOG.end_op();
        return Err("elf magic invalid");
    }

    /*
    let pgt = match PageTable::alloc_user_page_table(p.tf as usize) {
        None => {
            drop(idata);
            drop(inode);
            LOG.end_op();
            return Err("cannot alloc new user page table");
        }
        Some(pgt) => pgt,
    };
    */

    // Load program into memory.

    drop(idata);
    drop(inode);
    LOG.end_op();

    Ok(())
}
