/// An inode describes a single unnamed file.
/// The inode disk structure holds metadata:
/// rhe file's type, its size, the number of links referring to it
/// and the list of blocks holding the file's content.
///
/// The inodes are laid out sequentially on disk at SUPER_BLOCK.inodestart.
///
/// The kernel keeps a table of in-use inodes in memory
/// to provide a place for synchronizing access to inodes used by multiple processes.
/// The in-memory inodes include book-keeping information that is
/// not stored on disk: refcnt and valid.
///
/// the information in an node table entry is only correct when INodeData->valid is some.
/// ilock() reads the inode from the disk and sets INodeData->valid
/// while iput() clears INodeData->valid if refcnt has fallen to zero.
///
/// a typical sequence is:
///     let ip = INODE_TABLE.iget(dev, inum);
///     let guard = ip.ilock();
///     // examine and modify guard->xxx ...
///     drop(guard); // iunlock()
///     // and iput() when ip is dropped
///
/// ilock() is separate from iget() so that ststem calls can
/// get a long-term reference to an inode (as for an open file)
/// and only lock it for shot periods (e.g. in read()).
/// The separation helps avoid deadlock and races during pathname lookup.
/// iget() increments refcnt so that the inode stays in the table and pointers to it remain valid.
use core::{mem, ptr};

use array_macro::array;

use crate::{
    bio::{BCACHE, BSIZE},
    cpu::CPU_TABLE,
    log::{Log, LOG},
    param::ROOTDEV,
    println,
    sleeplock::{SleepLock, SleepLockGuard},
    spinlock::SpinLock,
};

pub unsafe fn init(dev: u32) {
    read_super_block(dev);
    LOG.init(dev, &SB);

    println!("fs: init done");
}

const NINODE: usize = 50;
const ROOTINO: u32 = 1;
const DIRSIZ: usize = 14;
pub static INODE_TABLE: INodeTable = INodeTable::new();

pub struct INodeTable {
    info: SpinLock<[INodeInfo; NINODE]>,
    data: [SleepLock<INodeData>; NINODE],
}

impl INodeTable {
    pub const fn new() -> Self {
        Self {
            info: SpinLock::new(array![_ => INodeInfo::new(); NINODE]),
            data: array![_ => SleepLock::new(INodeData::new()); NINODE],
        }
    }

    /// Find the inode with number inum on device dev
    /// and return the in-memory copy.
    /// does not lock the inode and does not read it from disk.
    pub fn iget(&self, dev: u32, inum: u32) -> INode {
        let mut guard = self.info.lock();
        let mut empty: Option<usize> = None;
        for (i, ip) in guard.iter_mut().enumerate() {
            if ip.dev == dev && ip.inum == inum {
                ip.refcnt += 1;
                drop(guard);
                return INode {
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

        INode {
            dev,
            inum,
            index: i,
        }
    }

    // drop a reference to an in-memory inode.
    // If that was the last reference, the inode table entry can be recycled.
    // If that was the last reference and the inode has no links to it,
    // free the inode (and its content) on disk.
    // All calls to iput() must be inside a transaction in case it has to free the inode.
    pub fn iput(&self, index: usize) {
        let mut guard = self.info.lock();

        if guard[index].refcnt == 1 {
            // refcnt == 1 means no other process can have the inode locked,
            // so this sleep-lock won't block/deadlock.
            let mut data_guard = self.data[index].lock();
            if data_guard.valid.is_some() && data_guard.dinode.nlink == 0 {
                // inode has no links and no other references
                // truncate and free

                drop(guard);

                // TODO: ip.itrunc();
                data_guard.dinode.typed = INodeType::Empty;
                // TODO: ip.iupdate();
                data_guard.valid.take();
                drop(data_guard);

                guard = self.info.lock();
            } else {
                drop(data_guard);
            }
        }

        guard[index].refcnt -= 1;
        drop(guard);
    }

    pub fn idup(&self, ip: &INode) -> INode {
        let mut guard = self.info.lock();
        let i = ip.index;
        guard[i].refcnt += 1;
        INode {
            dev: guard[i].dev,
            inum: guard[i].inum,
            index: i,
        }
    }

    pub fn namex(&self, path: &[u8], name: &mut [u8; DIRSIZ], parent: bool) -> Option<INode> {
        let mut ip = if path[0] == b'/' {
            self.iget(ROOTDEV, ROOTINO)
        } else {
            let cwd = unsafe { CPU_TABLE.my_proc().data.get_mut().cwd.as_ref().unwrap() };
            self.idup(cwd)
        };
        let mut cur = 0;
        loop {
            cur = self.skip_elem(path, cur, name);
            if cur == 0 {
                break;
            }

            let data_guard = ip.ilock();

            if data_guard.dinode.typed != INodeType::Directory {
                drop(data_guard);
                return None;
            }

            if parent && path[cur] == 0 {
                // Stop one level early.
                drop(data_guard);
                return Some(ip);
            }

            match data_guard.dirlookup(name) {
                Some(next) => {
                    drop(data_guard);
                    ip = next;
                }
                None => {
                    drop(data_guard);
                    return None;
                }
            }
        }

        Some(ip)
    }

    pub fn namei(&self, path: &[u8]) -> Option<INode> {
        let mut name: [u8; DIRSIZ] = [0; DIRSIZ];
        self.namex(path, &mut name, false)
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

struct INodeInfo {
    dev: u32,
    inum: u32,
    refcnt: usize,
}

impl INodeInfo {
    const fn new() -> Self {
        Self {
            dev: 0,
            inum: 0,
            refcnt: 0,
        }
    }
}

pub struct INodeData {
    valid: Option<(u32, u32)>, // (dev, inum)
    dinode: DiskINode,
}

impl INodeData {
    const fn new() -> Self {
        Self {
            valid: None,
            dinode: DiskINode::new(),
        }
    }

    // Look for a directory entry in a directory.
    fn dirlookup(&self, name: &mut [u8; DIRSIZ]) -> Option<INode> {
        let (dev, _) = self.valid.as_ref().unwrap();
        if self.dinode.typed != INodeType::Directory {
            panic!("dirlookup not DIR");
        }

        let de_size = mem::size_of::<DirEnt>();
        let mut de = DirEnt::empty();
        for off in (0..self.dinode.size).step_by(de_size) {
            // readi

            if de.inum == 0 {
                continue;
            }

            for i in 0..DIRSIZ {
                if de.name[i] != name[i] {
                    break;
                }
                if de.name[i] == 0 {
                    return Some(INODE_TABLE.iget(*dev, de.inum as u32));
                }
            }
        }

        None
    }

    fn readi() {}
}

impl INode {
    /// Lock the inode.
    /// Reads the inode from the disk if necessary.
    pub fn ilock(&self) -> SleepLockGuard<INodeData> {
        let mut guard = INODE_TABLE.data[self.index].lock();

        if guard.valid.is_some() {
            return guard;
        }

        let bp = unsafe { BCACHE.bread(self.dev, SB.inode_block(self.inum)) };
        let dip = unsafe { (bp.data_ptr() as *const DiskINode).offset(inode_offset(self.inum)) };
        guard.dinode = unsafe { dip.as_ref().unwrap().clone() };
        drop(bp);
        guard.valid = Some((self.dev, self.inum));

        if guard.dinode.typed == INodeType::Empty {
            panic!("ilock: no type");
        }

        guard
    }
}

impl Drop for INode {
    fn drop(&mut self) {
        INODE_TABLE.iput(self.index);
    }
}

const NDIRECT: usize = 12;
const NINDIRECT: usize = BSIZE / mem::size_of::<u32>();
const MAXFILE: usize = NDIRECT + NINDIRECT;

/// On disk inode structure
#[repr(C)]
#[derive(Clone, Copy)]
struct DiskINode {
    typed: INodeType,          // file type
    major: u16,                // major device number (Device Type only)
    minor: u16,                // minor device number (Device Type only)
    nlink: u16,                // number of links to inode in file system
    size: u32,                 // size of file (bytes)
    addrs: [u32; NDIRECT + 1], // data blocks addresses
}

impl DiskINode {
    const fn new() -> Self {
        Self {
            typed: INodeType::Empty,
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
enum INodeType {
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

pub struct INode {
    dev: u32,
    inum: u32,
    index: usize,
}

// number of inodes in a single block
const IPB: usize = BSIZE / mem::size_of::<DiskINode>();

#[inline]
fn inode_offset(inum: u32) -> isize {
    (inum as usize % IPB) as isize
}

pub static mut SB: SuperBlock = SuperBlock::new();
const FSMAGIC: u32 = 0x10203040;

unsafe fn read_super_block(dev: u32) {
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
    size: u32,
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

    fn inode_block(&self, inum: u32) -> u32 {
        inum / u32::try_from(IPB).unwrap() + self.inodestart
    }
}
