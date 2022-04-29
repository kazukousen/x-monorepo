//! Term of `inode` can have two related meanings. On-disk data structure and In-memory.
//!
//! An inode describes a single unnamed file.
//! The inode disk structure holds metadata:
//! rhe file's type, its size, the number of links referring to it
//! and the list of blocks holding the file's content.
//!
//! The inodes are laid out sequentially on disk at SUPER_BLOCK.inodestart.
//!
//! The on-disk inode is structured by a `struct DiskInode`.
//!
//! The kernel keeps a table of in-use inodes in memory called `INODE_TABLE`
//! to provide a place for synchronizing access to inodes used by multiple processes.
//! The in-memory inodes include book-keeping information that is
//! not stored on disk: `refcnt` and `valid`.
//! The `refcnt` field counts the number of instances referring to the in-memory inode,
//! and kernel discards the inode from memory if the reference count drops to zero.
//! The `iget()` and `iput()` acquire and release an instance referring to an inode, modifying the reference count.
//!
//! the information in an node table entry is only correct when `valid` is some.
//! ilock() reads the inode from the disk and sets `valid`
//! while iput() clears `valid` if refcnt has fallen to zero.
//!
//! a typical sequence is:
//!
//!     let inode = INODE_TABLE.iget(dev, inum); // iget()
//!     let idata = inode.ilock(); // ilock()
//!     // examine and modify idata->xxx ...
//!     drop(idata); // iunlock()
//!     drop(inode); // iput()
//!
//! ilock() is separate from iget() so that system calls can
//! get a long-term reference to an inode (as for an open file)
//! and only lock it for shot periods (e.g. in read()).
//! The separation helps avoid deadlock and races during pathname lookup.
//! multiple process can hold an instance of an inode retuned by iget(),
//! but only one process can lock the inode at time.
//! iget() increments refcnt so that the inode stays in the table and pointers to it remain valid.

use core::{cmp::min, mem, ptr};

use array_macro::array;

use crate::{
    bio::{BCACHE, BSIZE},
    bmap,
    cpu::CPU_TABLE,
    log::LOG,
    param::ROOTDEV,
    println,
    proc::either_copy_out,
    sleeplock::{SleepLock, SleepLockGuard},
    spinlock::SpinLock,
    superblock::{read_super_block, SB},
};

pub unsafe fn init(dev: u32) {
    read_super_block(dev);
    LOG.init(dev, &SB);

    println!("fs: init done");
}

const NINODE: usize = 50;
const ROOTINO: u32 = 1;
const DIRSIZ: usize = 14;
pub static INODE_TABLE: InodeTable = InodeTable::new();

pub struct InodeTable {
    meta: SpinLock<[InodeMeta; NINODE]>,
    data: [SleepLock<InodeData>; NINODE],
}

impl InodeTable {
    pub const fn new() -> Self {
        Self {
            meta: SpinLock::new(array![_ => InodeMeta::new(); NINODE]),
            data: array![_ => SleepLock::new(InodeData::new()); NINODE],
        }
    }

    /// Find the inode with number inum on device dev
    /// and return the in-memory copy.
    /// does not lock the inode and does not read it from disk.
    pub fn iget(&self, dev: u32, inum: u32) -> Inode {
        let mut guard = self.meta.lock();
        let mut empty: Option<usize> = None;
        for (i, ip) in guard.iter_mut().enumerate() {
            if ip.dev == dev && ip.inum == inum {
                ip.refcnt += 1;
                drop(guard);
                return Inode {
                    dev,
                    inum,
                    index: i,
                };
            }

            if empty.is_none() && ip.refcnt == 0 {
                empty = Some(i);
            }
        }

        if empty.is_none() {
            panic!("iget: no inodes");
        }

        let i = empty.unwrap();
        guard[i].dev = dev;
        guard[i].inum = inum;
        guard[i].refcnt += 1;

        drop(guard);
        Inode {
            dev,
            inum,
            index: i,
        }
    }

    /// Drop a reference to an in-memory inode.
    /// if that was the last reference, the inode table entry can be recycled.
    /// if that was the last reference and the inode has no links to it,
    /// free the inode (and its content) on disk.
    /// all calls to iput() must be inside a transaction in case it has to free the inode.
    pub fn iput(&self, index: usize) {
        let mut guard = self.meta.lock();

        if guard[index].refcnt == 1 {
            // refcnt == 1 means no other process can have the inode locked,
            // so this sleep-lock won't block/deadlock.
            let mut data_guard = self.data[index].lock();
            if data_guard.valid.is_some() && data_guard.dinode.nlink == 0 {
                // inode has no links and no other references
                // truncate and free

                drop(guard);

                data_guard.itrunc();
                data_guard.dinode.typ = InodeType::Empty;
                data_guard.iupdate();
                data_guard.valid.take();
                drop(data_guard);

                guard = self.meta.lock();
            } else {
                drop(data_guard);
            }
        }

        guard[index].refcnt -= 1;
        drop(guard);
    }

    pub fn idup(&self, ip: &Inode) -> Inode {
        let mut guard = self.meta.lock();
        let i = ip.index;
        guard[i].refcnt += 1;
        Inode {
            dev: guard[i].dev,
            inum: guard[i].inum,
            index: i,
        }
    }

    /// Allocate an inode on device dev.
    /// Mark it as allocated by giving it type type.
    /// Returns an unlocked but allocated and referenced inode.
    ///
    /// it panics if the table have no inodes.
    fn ialloc(&self, dev: u32, typ: InodeType) -> Inode {
        for inum in 1..unsafe { SB.ninodes } {
            let mut buf = BCACHE.bread(dev, inum);
            let dinode_ptr =
                unsafe { (buf.data_ptr_mut() as *mut DiskInode).offset(inode_offset(inum)) };
            let mut dinode = unsafe { dinode_ptr.as_mut().unwrap() };
            if dinode.typ == InodeType::Empty {
                // found a free inode
                unsafe { ptr::write_bytes(dinode_ptr, 0, 1) };
                dinode.typ = typ;
                // mark it allocated on the disk
                LOG.write(&mut buf);
                drop(buf);
                return self.iget(dev, inum);
            }
            drop(buf);
        }

        panic!("ialloc: no inodes");
    }

    // NOTE: returning an unlocked is correct?
    pub fn create(
        &self,
        path: &[u8],
        typ: InodeType,
        major: u16,
        minor: u16,
    ) -> Result<Inode, &'static str> {
        let mut name = [0u8; DIRSIZ];
        let dir = self.nameiparent(&path, &mut name).ok_or("create: parent")?;
        let mut dirdata = dir.ilock();

        if let Some(inode) = dirdata.dirlookup(&name) {
            // reuse?
            let idata = inode.ilock();
            if typ == InodeType::File
                && (idata.dinode.typ == InodeType::File || idata.dinode.typ == InodeType::Device)
            {
                drop(idata);
                return Ok(inode);
            }
            drop(idata);
            return Err("create: already exists and cannot reuse");
        }

        let inode = self.ialloc(dir.dev, typ);
        let mut idata = inode.ilock();
        idata.dinode.major = major;
        idata.dinode.minor = minor;
        idata.dinode.nlink = 1;
        idata.iupdate();

        if typ == InodeType::Directory {
            // Create . and .. entries.
            dirdata.dinode.nlink += 1; // for ".."
            dirdata.iupdate();
            // No nlink++ for "." because avoid cyclic ref count.
            let name: [u8; DIRSIZ] = [b'.', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            idata
                .dirlink(&name, inode.inum)
                .or(Err("create: create dot '.'"))?;
            let name: [u8; DIRSIZ] = [b'.', b'.', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            idata
                .dirlink(&name, inode.inum)
                .or(Err("create: create dot '..'"))?;
        }
        drop(idata);

        dirdata
            .dirlink(&name, inode.inum)
            .or(Err("create: dirlink"))?;
        drop(dirdata);
        drop(dir);

        Ok(inode)
    }

    /// if the path begins with a slash, evalution begins at the root, otherwise, the current
    /// directory.
    ///
    /// The procedure `namex` may take a long time to complete:
    ///     it could involve several disk operations to read inodes and directory blocks for the
    ///     directories traversed in the pathname (if they are not in the buffer cache).
    /// xv6 is carefully designed so that if invocation of `namex` by one kernel thread is blocked
    /// on a disk I/O, another kernel thread locking up a different pathname can proceed
    /// concurrency. `namex` locks each directory in the path separately so that lookups in
    /// different directories can proceed in parallel.
    ///
    /// The concurrency introduces some challenges. for example, while one kernel thread is locking
    /// up a pathname another kernel thread may be changing the directory tree unlinking a
    /// directory.
    /// A potential risk is that a lookup may be searching a directory that has been deleted by
    /// another kernel thread and its blocks have been re-used for another directory or file.
    /// `xv6` avoids such races. for example, when executing `dirlookup` in `namex`, the lookup
    /// thread holds the lock on the directory and `dirlookup` returns an inode that was obtained
    /// using `iget`. `iget` increases the reference count of the inode.
    /// only after receiving the inode from `dirlookup` does `namex` release the lock on the
    /// directory. now another thread may unlink the inode from the directory but xv6 will not
    /// delete the inode yet, because the reference count of the inode is still larger than zero.
    /// Another risk is deadlock. for example, `next` points to the same inode as `inode` when
    /// locking up ".". locking `next` before releasing the lock on `inode` would result in a
    /// deadlock. to avoid this deadlock, `namex` unlocks the directory before obtaining a lock on
    /// `next`. here again we see why the separation between `iget` and `ilock` is important.
    pub fn namex(&self, path: &[u8], name: &mut [u8; DIRSIZ], parent: bool) -> Option<Inode> {
        let mut inode = if path[0] == b'/' {
            self.iget(ROOTDEV, ROOTINO)
        } else {
            let cwd = unsafe { CPU_TABLE.my_proc().data.get_mut().cwd.as_ref().unwrap() };
            self.idup(cwd)
        };
        let mut path_pos = 0;
        loop {
            path_pos = self.skip_elem(path, path_pos, name);
            if path_pos == 0 {
                break;
            }

            // inode type is not guaranteed to have been loaded from disk until `ilock` runs.
            let mut data_guard = inode.ilock();

            if data_guard.dinode.typ != InodeType::Directory {
                drop(data_guard);
                return None;
            }

            if parent && path[path_pos] == 0 {
                // Stop one level early.
                drop(data_guard);
                return Some(inode);
            }

            match data_guard.dirlookup(name) {
                Some(next) => {
                    drop(data_guard);
                    inode = next;
                }
                None => {
                    drop(data_guard);
                    return None;
                }
            }
        }

        Some(inode)
    }

    /// Lookup and return the inode for a pathname.
    /// must be called inside a transaction (begin_op/end_op) since it calls iput().
    pub fn namei(&self, path: &[u8]) -> Option<Inode> {
        let mut name: [u8; DIRSIZ] = [0; DIRSIZ];
        self.namex(path, &mut name, false)
    }

    pub fn nameiparent(&self, path: &[u8], name: &mut [u8; DIRSIZ]) -> Option<Inode> {
        self.namex(path, name, true)
    }

    /// Copy the next path element from path into name.
    /// Return the offset following the copied one.
    /// Examples:
    ///     skip_elem("a/bb/c", name) = 1, setting name = "a"
    ///     skip_elem("///a//bb", name) = 5, setting name = "a"
    ///     skip_elem("a", name) = 0, setting name = "a"
    ///     skip_elem("", name) = skip_elem("////", name) = 0
    fn skip_elem(&self, path: &[u8], mut cur: usize, name: &mut [u8; DIRSIZ]) -> usize {
        while path[cur] == b'/' {
            cur += 1;
        }
        if path[cur] == 0 {
            return 0;
        }

        let s = cur;

        while path[cur] != b'/' && path[cur] != 0 {
            cur += 1;
        }

        let mut len = cur - s;

        if len >= name.len() {
            len = name.len() - 1;
        }
        unsafe {
            ptr::copy_nonoverlapping(path.as_ptr().offset(s as isize), name.as_mut_ptr(), len);
        }
        name[len] = 0;

        while path[cur] == b'/' {
            cur += 1;
        }

        return cur;
    }
}

impl Inode {
    /// Lock the inode.
    /// Reads the inode from the disk if necessary.
    pub fn ilock(&self) -> SleepLockGuard<InodeData> {
        let mut guard = INODE_TABLE.data[self.index].lock();

        if guard.valid.is_some() {
            return guard;
        }

        // load on-disk structure inode.
        let buf = unsafe { BCACHE.bread(self.dev, SB.inode_block(self.inum)) };
        let dinode =
            unsafe { (buf.data_ptr() as *const DiskInode).offset(inode_offset(self.inum)) };
        guard.dinode = unsafe { dinode.as_ref().unwrap().clone() };
        drop(buf);

        if guard.dinode.typ == InodeType::Empty {
            panic!("ilock: no type");
        }

        guard.valid = Some((self.dev, self.inum));
        guard
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        INODE_TABLE.iput(self.index);
    }
}

struct InodeMeta {
    dev: u32,
    inum: u32,
    refcnt: usize,
}

impl InodeMeta {
    const fn new() -> Self {
        Self {
            dev: 0,
            inum: 0,
            refcnt: 0,
        }
    }
}

/// it is always protected by sleep-lock.
pub struct InodeData {
    valid: Option<(u32, u32)>, // (dev, inum)
    dinode: DiskInode,
}

/// The on-disk inode structure `DiskInode`, contains a size and an array of block numbers.
/// The inode data is found in the blocks listed in the `DiskInode`'s `addrs` field array.
/// The first 12kB(NDIRECT x BSIZE) of a file can be loaded from the blocks listed in the inode,
/// while the next 256kB (NINDIRECT x BSIZE) can only be loaded after consulting the indirect
/// blocks.
/// (This is a good on-disk representation but a complex one for clients...)
impl InodeData {
    const fn new() -> Self {
        Self {
            valid: None,
            dinode: DiskInode::new(),
        }
    }

    /// Truncate inode (discard contents).
    /// Caller must hold sleep-lock.
    fn itrunc(&mut self) {
        let (dev, _) = self.valid.unwrap();

        // direct blocks
        for i in 0..NDIRECT {
            if self.dinode.addrs[i] > 0 {
                bmap::free(dev, self.dinode.addrs[i]);
                self.dinode.addrs[i] = 0;
            }
        }

        // an indirect block
        if self.dinode.addrs[NDIRECT] > 0 {
            let buf = BCACHE.bread(dev, self.dinode.addrs[NDIRECT]);
            let bn_ptr = buf.data_ptr() as *const u32;
            for i in 0..(NINDIRECT as isize) {
                let bn = unsafe { ptr::read(bn_ptr.offset(i)) };
                if bn != 0 {
                    bmap::free(dev, bn);
                }
            }
            drop(buf);
            bmap::free(dev, self.dinode.addrs[NDIRECT]);
            self.dinode.addrs[NDIRECT] = 0;
        }

        self.dinode.size = 0;
        self.iupdate();
    }

    /// Copy a modified in-memory inode to disk.
    /// Must be called after every change to itself dinode field
    /// that lives on disk.
    /// Caller must hold sleep-lock.
    fn iupdate(&mut self) {
        let (dev, inum) = self.valid.unwrap();
        let mut bp = unsafe { BCACHE.bread(dev, SB.inode_block(inum)) };
        let dip = unsafe { (bp.data_ptr() as *mut DiskInode).offset(inode_offset(inum)) };
        unsafe { ptr::write(dip, self.dinode) };
        LOG.write(&mut bp);
    }

    /// Look for a directory entry in a directory.
    pub fn dirlookup(&mut self, name: &[u8; DIRSIZ]) -> Option<Inode> {
        let (dev, _) = self.valid.unwrap();
        if self.dinode.typ != InodeType::Directory {
            panic!("dirlookup not DIR");
        }

        let de_size = mem::size_of::<DirEnt>();
        let mut de = DirEnt::empty();
        let de_ptr = &mut de as *mut DirEnt as *mut u8;
        for off in (0..self.dinode.size).step_by(de_size) {
            self.readi(false, de_ptr, off as usize, de_size)
                .expect("dirlookup: read");

            if de.inum == 0 {
                continue;
            }

            for i in 0..DIRSIZ {
                if de.name[i] != name[i] {
                    break;
                }
                if de.name[i] == 0 {
                    return Some(INODE_TABLE.iget(dev, de.inum as u32));
                }
            }
        }

        None
    }

    /// Returns the disk block number of the offset'th data block in the inode.
    /// If there is no such block yet, bmap() allocates one.
    fn bmap(&mut self, mut offset: usize) -> u32 {
        let (dev, _) = self.valid.unwrap();

        if offset < NDIRECT {
            if self.dinode.addrs[offset] != 0 {
                return self.dinode.addrs[offset];
            }
            let bn = bmap::alloc(dev);
            self.dinode.addrs[offset] = bn;
            return bn;
        }

        offset -= NDIRECT;

        if offset < NINDIRECT {
            // load the indirect block, allocating if necessary.
            let indirect_bn = if self.dinode.addrs[NDIRECT] != 0 {
                self.dinode.addrs[NDIRECT]
            } else {
                let bn = bmap::alloc(dev);
                self.dinode.addrs[NDIRECT] = bn;
                bn
            };
            let mut buf = BCACHE.bread(dev, indirect_bn);

            let bn_ptr = unsafe { (buf.data_ptr_mut() as *mut u32).offset(offset as isize) };
            let bn = unsafe { ptr::read(bn_ptr) };
            if bn == 0 {
                let freed = bmap::alloc(dev);
                unsafe { ptr::write(bn_ptr, freed) };
                LOG.write(&mut buf);
            }
            drop(buf);
            return bn;
        }

        panic!("bmap: out of range");
    }

    /// Read data from inode.
    pub fn readi(
        &mut self,
        is_user: bool,
        mut dst: *mut u8,
        mut offset: usize,
        mut count: usize,
    ) -> Result<(), ()> {
        let (dev, _) = self.valid.unwrap();

        let end = offset.checked_add(count).ok_or(())?;
        if end > self.dinode.size as usize {
            return Err(());
        }

        // copy the file to dst by separating it into multiparts.
        // [offset:BSIZE], [BSIZE:BSIZE*2], [BSIZE*N:count]
        while count > 0 {
            let read_count = min(BSIZE - offset % BSIZE, count);
            let buf = BCACHE.bread(dev, self.bmap(offset / BSIZE));
            let src_ptr =
                unsafe { (buf.data_ptr() as *const u8).offset((offset % BSIZE) as isize) };
            either_copy_out(is_user, dst, src_ptr, read_count);
            drop(buf);
            offset += read_count;
            count -= read_count;
            dst = unsafe { dst.offset(read_count as isize) };
        }

        Ok(())
    }

    fn writei(
        &mut self,
        is_user: bool,
        mut src: *const u8,
        mut offset: usize,
        mut count: usize,
    ) -> Result<(), ()> {
        // TODO
        Err(())
    }

    /// Write a new directory entry (name, inum) into the directory this.
    fn dirlink(&mut self, name: &[u8; DIRSIZ], inum: u32) -> Result<(), ()> {
        // Check that name is not present.
        if let Some(inode) = self.dirlookup(&name) {
            drop(inode);
            return Err(());
        }

        // Look for an empty DirEnt.
        let de_size = mem::size_of::<DirEnt>();
        let mut de = DirEnt::empty();
        let de_ptr = &mut de as *mut DirEnt as *mut u8;
        let mut offset = 0;
        for off in (0..self.dinode.size as usize).step_by(de_size) {
            self.readi(false, de_ptr, off, de_size)?;
            if de.inum == 0 {
                offset = off;
                break;
            }
        }

        for i in 0..DIRSIZ {
            de.name[i] = name[i];
            if name[i] == 0 {
                break;
            }
        }
        de.inum = inum.try_into().unwrap();

        self.writei(false, de_ptr as *const u8, offset, de_size)
    }
}

const NDIRECT: usize = 12;
const NINDIRECT: usize = BSIZE / mem::size_of::<u32>();
const MAXFILE: usize = NDIRECT + NINDIRECT;

/// On disk inode structure
#[repr(C)]
#[derive(Clone, Copy)]
struct DiskInode {
    typ: InodeType,            // file type
    major: u16,                // major device number (Device Type only)
    minor: u16,                // minor device number (Device Type only)
    nlink: u16,                // number of directory entries that refer to a file
    size: u32,                 // size of file (bytes)
    addrs: [u32; NDIRECT + 1], // data blocks addresses
}

impl DiskInode {
    const fn new() -> Self {
        Self {
            typ: InodeType::Empty,
            major: 0,
            minor: 0,
            nlink: 0,
            size: 0,
            addrs: [0; NDIRECT + 1],
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InodeType {
    Empty = 0,
    Directory = 1,
    File = 2,
    Device = 3,
}

#[repr(C)]
struct DirEnt {
    inum: u16,
    name: [u8; DIRSIZ],
}

impl DirEnt {
    const fn empty() -> Self {
        Self {
            inum: 0,
            name: [0; DIRSIZ],
        }
    }
}

pub struct Inode {
    dev: u32,
    inum: u32,
    index: usize,
}

// number of inodes in a single block
pub const IPB: usize = BSIZE / mem::size_of::<DiskInode>();

#[inline]
fn inode_offset(inum: u32) -> isize {
    (inum as usize % IPB) as isize
}
