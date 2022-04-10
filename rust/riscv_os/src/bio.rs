use core::ptr;

use array_macro::array;

use crate::{spinlock::SpinLock, param::NBUF};


/// The buffer cache is a linked list of buf structures holding
/// cached copies of disk block contents.
/// Caching disk blocks in memory reduces the number of disk reads
/// and also provides a synchronization point for disk blocks used by multiple processes.

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
        lru.tail = &mut lru.inner[n-1];

        lru.inner[0].prev = ptr::null_mut();
        lru.inner[0].next = &mut lru.inner[1];
        lru.inner[n-1].prev = &mut lru.inner[n-2];
        lru.inner[n-1].next = ptr::null_mut();
        for i in 1..(n-1) {
            lru.inner[i].prev = &mut lru.inner[i-1];
            lru.inner[i].next = &mut lru.inner[i+1];
        }
    }
}

struct BufLru {
    inner: [Buf; NBUF],
    head: *mut Buf,
    tail: *mut Buf,
}

// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for BufLru {}

impl BufLru {
    const fn new() -> Self {
        Self {
            inner: array![_ => Buf::new(); NBUF],
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }
}

struct Buf {
    dev: u32,
    blockno: u32,
    refcnt: usize,
    prev: *mut Buf,
    next: *mut Buf,
}

impl Buf {
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

