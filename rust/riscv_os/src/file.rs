//! A cool aspect of the Unix interface is that most resources in Unix are represented as files,
//! including devices such as the console, pipes, and of course, real files. The file descriptor
//! layer is the layer that archives this uniformity.

use core::{cell::UnsafeCell, cmp::min};

use alloc::sync::Arc;

use crate::{
    bio::BSIZE,
    fs::{Inode, InodeType, INODE_TABLE},
    log::LOG,
    param::MAXOPBLOCKS,
};

pub const O_RDONLY: i32 = 0x000;
pub const O_WRONLY: i32 = 0x001;
pub const O_RDWR: i32 = 0x002;
pub const O_CREATE: i32 = 0x200;
pub const O_TRUNC: i32 = 0x400;

/// Each open file is represented by a `struct File`, which is a wrapper around either an inode or
/// a pipe, plus an I/O offset.
/// each call to `open` creates a new open file (a new `struct File`):
///     if multiple processes open the same file independently, the different instances will have
///     different I/O offsets.
pub struct File {
    // A file can be open for reading or writing or both. The `readable` and `writable` fields
    // track this.
    readable: bool,
    writable: bool,

    inner: FileInner,
}

impl File {
    /// must be called in a log transaction.
    pub fn open(path: &[u8], o_mode: i32) -> Option<Arc<Self>> {
        LOG.begin_op();

        let inode = if o_mode & O_CREATE > 0 {
            INODE_TABLE.create(path, InodeType::File, 0, 0).ok()
        } else {
            INODE_TABLE.namei(path)
        }
        .or_else(|| {
            LOG.end_op();
            None
        })?;

        let readable = o_mode & O_WRONLY == 0;
        let writable = (o_mode & O_WRONLY > 0) || (o_mode & O_RDWR > 0);

        // we designed that `create` and `namei` does not return a locked inode.
        // so `open` must lock the inode itself.
        let mut idata = inode.ilock();

        let inner = match idata.get_type() {
            InodeType::Empty => panic!("create: inode empty"),
            InodeType::Directory => {
                drop(idata);
                if o_mode != O_RDONLY {
                    drop(inode);
                    LOG.end_op();
                    return None;
                }
                FileInner::Inode(FileInode {
                    inode: Some(inode),
                    offset: UnsafeCell::new(0),
                })
            }
            InodeType::File => {
                if o_mode & O_TRUNC > 0 {
                    idata.itrunc();
                }
                drop(idata);
                FileInner::Inode(FileInode {
                    inode: Some(inode),
                    offset: UnsafeCell::new(0),
                })
            }
            InodeType::Device => panic!("open: device type"),
        };
        LOG.end_op();

        Some(Arc::new(Self {
            readable,
            writable,
            inner,
        }))
    }

    pub fn fwrite(&self, mut addr: *const u8, n: usize) -> Result<usize, &'static str> {
        if !self.writable {
            return Err("fwrite: not writable");
        }

        match &self.inner {
            FileInner::Inode(fi) => {
                // write a few blocks at a time to avoid exceeding the maximum log transaction
                // size, including i-node, indirect block, allocation blocks, and 2 blocks of slop
                // for non-aligned writes. this really belongs lower down, since writei() might be
                // writing a device like the console.
                let max_n = ((MAXOPBLOCKS - 1 - 1 - 2) / 2) * BSIZE;
                let offset = unsafe { &mut *fi.offset.get() };

                let inode = fi.inode.as_ref().unwrap();
                let mut idata = inode.ilock();
                LOG.begin_op();
                for i in (0..n).step_by(max_n) {
                    let write_n = min(max_n, n - i);
                    if idata.writei(true, addr, *offset, write_n).is_err() {
                        drop(idata);
                        LOG.end_op();
                        return Err("fwrite: inode type");
                    };
                    *offset += write_n;
                    addr = unsafe { addr.offset(write_n as isize) };
                }
                drop(idata);
                LOG.end_op();
                return Ok(n);
            }
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        match self.inner {
            FileInner::Inode(ref mut inner) => {
                LOG.begin_op();
                let inode = inner.inode.take();
                drop(inode);
                LOG.end_op();
            }
        }
    }
}

enum FileInner {
    Inode(FileInode),
}

struct FileInode {
    offset: UnsafeCell<usize>,
    inode: Option<Inode>,
}
