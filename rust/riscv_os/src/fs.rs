use crate::{bio::BCACHE, println};

pub unsafe fn init(dev: u32) {

    read_super_block(dev);

    println!("fs: init done");
}

#[repr(C)]
struct SuperBlock {
    magic: u32,
    size: u32,
    nblocks: u32,
    ninodes: u32,
    nlog: u32,
    logstart: u32,
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
}

static mut SB: SuperBlock = SuperBlock::new();

unsafe fn read_super_block(dev: u32) {

    println!("super_block: reading ...");

    let bp = BCACHE.bread(dev, 1);

    ptr::copy_nonoverlapping(
        bp.data_ptr() as *const SuperBlock,
        &mut SB as *mut SuperBlock,
        1,
    );

    println!("super_block: magic: {}", SB.magic);

    drop(bp);
}
