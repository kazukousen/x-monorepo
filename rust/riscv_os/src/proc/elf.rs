use core::{mem, cmp};

use crate::{
    fs::{InodeData, INODE_TABLE},
    log::LOG,
    page_table::PageTable,
    param::PAGESIZE,
    println,
    proc::ProcessData,
    sleeplock::SleepLockGuard,
};

const MAGIC: u32 = 0x464C457F;

pub fn load(p: &ProcessData, path: &[u8]) -> Result<(), &'static str> {
    LOG.begin_op();

    let inode = match INODE_TABLE.namei(&path) {
        None => {
            LOG.end_op();
            return Err("cannot find inode by given path");
        }
        Some(inode) => inode,
    };

    let mut idata = inode.ilock();

    println!("sys_exec: size={}", idata.size());

    // read elf header
    let mut elfhdr = mem::MaybeUninit::<ELFHeader>::uninit();
    let elfhdr_ptr = elfhdr.as_mut_ptr() as *mut u8;
    match idata.readi(false, elfhdr_ptr, 0, mem::size_of::<ELFHeader>()) {
        Err(_) => {
            drop(idata);
            drop(inode);
            LOG.end_op();
            return Err("cannot read the elf file");
        }
        Ok(_) => {}
    }
    let elfhdr = unsafe { elfhdr.assume_init() };

    if elfhdr.magic != MAGIC {
        drop(idata);
        drop(inode);
        LOG.end_op();
        return Err("elf magic invalid");
    }

    let mut pgt = match PageTable::alloc_user_page_table(p.tf as usize) {
        None => {
            drop(idata);
            drop(inode);
            LOG.end_op();
            return Err("cannot alloc new user page table");
        }
        Some(pgt) => pgt,
    };

    let mut size = 0usize;

    // Load program into memory.
    let off_start = elfhdr.phoff as usize;
    let ph_size = mem::size_of::<ProgHeader>();
    let off_end = off_start + elfhdr.phnum as usize * ph_size;
    for off in (off_start..off_end).step_by(ph_size) {
        // read program header section
        let mut ph = mem::MaybeUninit::<ProgHeader>::uninit();
        let ph_ptr = ph.as_mut_ptr() as *mut u8;
        if idata.readi(false, ph_ptr, off, ph_size).is_err() {
            pgt.unmap_user_page_table(size);
            drop(idata);
            drop(inode);
            LOG.end_op();
            return Err("cannot read the program section");
        };
        let ph = unsafe { ph.assume_init() };

        size = match pgt.uvm_alloc(size, (ph.vaddr + ph.memsz) as usize) {
            Err(msg) => {
                pgt.unmap_user_page_table(size);
                drop(idata);
                drop(inode);
                LOG.end_op();
                return Err(msg);
            }
            Ok(size) => size,
        };

        if let Err(msg) = load_segment(
            &mut pgt,
            &mut idata,
            ph.vaddr as usize,
            ph.off as usize,
            ph.filesz as usize,
        ) {
            return Err(msg);
        };
    }

    drop(idata);
    drop(inode);
    LOG.end_op();

    let oldsz = p.sz;

    // Allocate two pages.
    // Use the second as the user stack.
    size = match pgt.uvm_alloc(size, size + PAGESIZE*2) {
        Err(msg) => {
            pgt.unmap_user_page_table(size);
            return Err(msg);
        },
        Ok(size) => size,
    };

    pgt.unmap_user_page_table(size);

    Ok(())
}

fn load_segment(
    pgt: &mut PageTable,
    idata: &mut SleepLockGuard<'_, InodeData>,
    va: usize,
    offset: usize,
    sz: usize,
) -> Result<(), &'static str> {
    for i in (0..sz).step_by(PAGESIZE) {
        let pa = pgt.walk_addr(va + i)?;
        let n = cmp::min(sz - i, PAGESIZE);
        if idata.readi(false, pa as *mut u8, offset + i, n).is_err() {
            return Err("load_segment: cannot read the program segment");
        };
    }

    Ok(())
}

/// File header
#[repr(C)]
struct ELFHeader {
    magic: u32,
    elf: [u8; 12],
    typed: u16,
    machine: u16,
    version: u32,
    entry: u64,
    // program header position
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    // number of program headers
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

/// Program section header
#[repr(C)]
struct ProgHeader {
    typed: u32,
    flags: u32,
    off: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}
