use core::ptr;

use array_macro::array;

use crate::{param::NBUF, spinlock::SpinLock};

/// The buffer cache is a linked list of buf structures holding
/// cached copies of disk block contents.
/// Caching disk blocks in memory reduces the number of disk reads
/// and also provides a synchronization point for disk blocks used by multiple processes.

pub const BSIZE: usize = 1024; // size of disk block
pub static BCACHE: BCache = BCache::new();

pub struct BCache {
    lru: SpinLock<BufLru>,
}

impl BCache {
    const fn new() -> Self {
        Self {
            lru: SpinLock::new(BufLru::new()),
        }
    }

    pub fn init(&self) {
        let mut lru = self.lru.lock();
        let n = lru.inner.len();

        lru.head = &mut lru.inner[0];
        lru.tail = &mut lru.inner[n - 1];

        lru.inner[0].prev = ptr::null_mut();
        lru.inner[0].next = &mut lru.inner[1];
        lru.inner[n - 1].prev = &mut lru.inner[n - 2];
        lru.inner[n - 1].next = ptr::null_mut();
        for i in 1..(n - 1) {
            lru.inner[i].prev = &mut lru.inner[i - 1];
            lru.inner[i].next = &mut lru.inner[i + 1];
        }
    }

    fn bget(&self) {
        let mut lru = self.lru.lock();
    }
}

struct Buf {
    valid: bool,
    data: BufData,
}

#[repr(C, align(8))]
pub struct BufData([u8; BSIZE]);

struct BufLru {
    inner: [BufInfo; NBUF],
    head: *mut BufInfo, // most-recently-used
    tail: *mut BufInfo,
}

// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for BufLru {}

impl BufLru {
    const fn new() -> Self {
        Self {
            inner: array![_ => BufInfo::new(); NBUF],
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    // TODO: what return
    fn find(&self, dev: u32, blockno: u32) -> Option<()> {
        let mut b = self.head;

        while !b.is_null() {
            let buf = unsafe { b.as_mut().unwrap() };
            if buf.dev == dev && buf.blockno == blockno {
                buf.refcnt += 1;
                return Some(());
            }
            b = buf.next;
        }

        None
    }
}

struct BufInfo {
    dev: u32,
    blockno: u32,
    refcnt: usize,
    prev: *mut BufInfo,
    next: *mut BufInfo,
}

impl BufInfo {
    const fn new() -> Self {
        Self {
            dev: 0,
            blockno: 0,
            refcnt: 0,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}
