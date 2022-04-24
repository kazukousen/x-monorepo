use core::ptr;

use crate::{bio::BCACHE, fs::IPB};

pub static mut SB: SuperBlock = SuperBlock::new();
const FSMAGIC: u32 = 0x10203040;

pub unsafe fn read_super_block(dev: u32) {
    let bp = BCACHE.bread(dev, 1);

    ptr::copy_nonoverlapping(
        bp.data_ptr() as *const SuperBlock,
        &mut SB as *mut SuperBlock,
        1,
    );

    if SB.magic != FSMAGIC {
        panic!("invalid file system");
    }

    drop(bp);
}

#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub size: u32,
    nblocks: u32,
    ninodes: u32,
    pub nlog: u32,
    pub logstart: u32,
    inodestart: u32,
    bmapstart: u32,
}

impl SuperBlock {
    const fn new() -> Self {
        Self {
            magic: 0,
            size: 0,
            nblocks: 0,
            ninodes: 0,
            nlog: 0,
            logstart: 0,
            inodestart: 0,
            bmapstart: 0,
        }
    }

    pub fn inode_block(&self, inum: u32) -> u32 {
        inum / u32::try_from(IPB).unwrap() + self.inodestart
    }
}
