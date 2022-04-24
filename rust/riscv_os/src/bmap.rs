//! File and directory content is stored in disk blocks,
//! but for the disk blocks, kernel must allocate them from a free pool.
//! the block allocator maintains a free bitmap on disk, with one bit per block.
//! a zero bit indicates that the corresponding block is free / a one bit indicates that it is in
//! use.
//! The program `mkfs` sets the bits correspoinding to the boot sector, superblock, log blocks,
//! inode blocks, bit map blocks.

use core::ptr;

use crate::{
    bio::{BCACHE, BSIZE},
    log::LOG,
    superblock::SB,
};

const BPB: usize = BSIZE * 8; // Bits-Per-Block

/// Allocates a zeroed disk block.
/// looks for a block whose a bitmap bit is zero, indicating that it is free.
/// finds a such block, updates the bitmap bit and return the block.
pub fn alloc(dev: u32) -> u32 {
    let size = unsafe { SB.size } as usize;
    for base in (0..size).step_by(BPB) {
        let mut buf = BCACHE.bread(dev, unsafe { SB.inode_block(base as u32) });
        let buf_data = unsafe { buf.data_ptr_mut().as_mut().unwrap() };

        for offset in 0..BPB {
            if base + offset >= size {
                break;
            }

            let index = offset / 8; // at index of byte in the block (0..BSIZE)
            let bit = offset % 8; // a bit in a byte

            if buf_data[index] & (1 << bit) != 0 {
                // block is not free; already in use
                continue;
            }

            // mark block in use
            buf_data[index] = (buf_data[index] as usize | (1 << bit)).try_into().unwrap();

            let blockno: u32 = (base + offset).try_into().unwrap();

            LOG.write(&mut buf);
            drop(buf);
            bzero(dev, blockno);

            return blockno;
        }
        drop(buf);
    }

    panic!("balloc: out of blocks");
}

/// Frees a block.
pub fn free(dev: u32, blockno: u32) {
    // TODO:
}

// zero a block.
#[inline]
fn bzero(dev: u32, blockno: u32) {
    let mut buf = BCACHE.bread(dev, blockno);
    unsafe { ptr::write_bytes(buf.data_ptr_mut(), 0, 1) };
    LOG.write(&mut buf);
    drop(buf);
}
