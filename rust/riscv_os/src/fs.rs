use core::{mem, ptr};

use crate::{bio::BCACHE, println};

pub unsafe fn init(dev: u32) {
    read_super_block(dev);

    println!("fs: init done");
}

#[repr(C)]
struct SuperBlock {
    magic: usize,
    size: usize,
    nblocks: usize,
    ninodes: usize,
    nlog: usize,
    logstart: usize,
    inodestart: usize,
    bmapstart: usize,
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
}

static mut SB: SuperBlock = SuperBlock::new();

unsafe fn read_super_block(dev: u32) {
    println!("super_block: bread");
    let bp = BCACHE.bread(dev, 1);

    println!("super_block: copy");
    ptr::copy_nonoverlapping(
        bp.data_ptr() as *const SuperBlock,
        &mut SB as *mut _,
        mem::size_of::<SuperBlock>(),
    );

    println!("super_block: brelse");
    BCACHE.brelse(bp.index);
}
