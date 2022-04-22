use core::{
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{AtomicBool, Ordering},
};

use array_macro::array;

use crate::{
    param::NBUF,
    sleeplock::{SleepLock, SleepLockGuard},
    spinlock::SpinLock,
    virtio::DISK,
};

/// The buffer cache is a linked list of buf structures holding
/// cached copies of disk block contents.
/// Caching disk blocks in memory reduces the number of disk reads
/// and also provides a synchronization point for disk blocks used by multiple processes.

pub const BSIZE: usize = 1024; // size of disk block
pub static BCACHE: BCache = BCache::new();

pub struct BCache {
    lru: SpinLock<BufLru>,
    bufs: [Buf; NBUF],
}

impl BCache {
    const fn new() -> Self {
        Self {
            lru: SpinLock::new(BufLru::new()),
            bufs: array![_ => Buf::new(); NBUF],
        }
    }

    pub fn init(&self) {
        self.lru.lock().init();
    }

    pub fn bread(&self, dev: u32, blockno: u32) -> GuardBuf {
        let mut buf = self.bget(dev, blockno);

        if !self.bufs[buf.index].valid.load(Ordering::Relaxed) {
            DISK.rw(&mut buf, false);
            self.bufs[buf.index].valid.store(true, Ordering::Relaxed);
        }
        buf
    }

    pub fn brelse(&self, index: usize) {
        self.lru.lock().brelse(index);
    }

    fn bget(&self, dev: u32, blockno: u32) -> GuardBuf {
        let lru = self.lru.lock();

        if let Some((index, rc_ptr)) = lru.find(dev, blockno) {
            // found cached block
            drop(lru);
            return GuardBuf {
                index,
                dev,
                blockno,
                rc_ptr,
                data: Some(self.bufs[index].data.lock()),
            };
        }

        if let Some((index, rc_ptr)) = lru.recycle(dev, blockno) {
            // not cached block
            self.bufs[index].valid.store(false, Ordering::Relaxed);
            drop(lru);
            return GuardBuf {
                index,
                dev,
                blockno,
                rc_ptr,
                data: Some(self.bufs[index].data.lock()),
            };
        }

        panic!("bcache: no buffers");
    }
}

pub struct GuardBuf<'a> {
    index: usize,
    dev: u32,
    pub blockno: u32,
    rc_ptr: *mut usize,
    data: Option<SleepLockGuard<'a, BufData>>,
}

impl<'a> GuardBuf<'a> {
    pub fn data_ptr_mut(&mut self) -> *mut BufData {
        let guard = self.data.as_mut().unwrap();
        guard.deref_mut()
    }

    pub fn data_ptr(&self) -> *const BufData {
        let guard = self.data.as_ref().unwrap();
        guard.deref()
    }
}

impl<'a> GuardBuf<'a> {
    pub fn bwrite(&mut self) {
        DISK.rw(self, true);
    }

    pub unsafe fn bpin(&mut self) {
        self.rc_ptr.as_mut().map(|v| *v += 1);
    }

    pub unsafe fn bunpin(&mut self) {
        self.rc_ptr.as_mut().map(|v| *v -= 1);
    }
}

impl<'a> Drop for GuardBuf<'a> {
    fn drop(&mut self) {
        drop(self.data.take());
        BCACHE.brelse(self.index);
    }
}

struct Buf {
    valid: AtomicBool,
    data: SleepLock<BufData>,
}

impl Buf {
    const fn new() -> Self {
        Self {
            valid: AtomicBool::new(false),
            data: SleepLock::new(BufData::new()),
        }
    }
}

#[repr(C, align(8))]
pub struct BufData([u8; BSIZE]);

impl BufData {
    const fn new() -> Self {
        Self([0; BSIZE])
    }
}

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
            inner: array![i => BufInfo::new(i); NBUF],
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    fn init(&mut self) {
        let n = self.inner.len();
        self.head = &mut self.inner[0];
        self.tail = &mut self.inner[n - 1];

        self.inner[0].prev = ptr::null_mut();
        self.inner[0].next = &mut self.inner[1];
        self.inner[n - 1].prev = &mut self.inner[n - 2];
        self.inner[n - 1].next = ptr::null_mut();

        for i in 1..(n - 1) {
            self.inner[i].prev = &mut self.inner[i - 1];
            self.inner[i].next = &mut self.inner[i + 1];
        }
    }

    fn find(&self, dev: u32, blockno: u32) -> Option<(usize, *mut usize)> {
        let mut b = self.head;

        while !b.is_null() {
            let buf = unsafe { b.as_mut().unwrap() };
            if buf.dev == dev && buf.blockno == blockno {
                buf.refcnt += 1;
                return Some((buf.index, &mut buf.refcnt));
            }
            b = buf.next;
        }

        None
    }

    fn recycle(&self, dev: u32, blockno: u32) -> Option<(usize, *mut usize)> {
        let mut b = self.tail;

        while !b.is_null() {
            let buf = unsafe { b.as_mut().unwrap() };
            if buf.refcnt == 0 {
                buf.dev = dev;
                buf.blockno = blockno;
                buf.refcnt += 1;
                return Some((buf.index, &mut buf.refcnt));
            }
            b = buf.prev;
        }

        None
    }

    /// Release a locked buffer.
    /// If no live reference,
    /// Move the buffer to the head of the most-recently-used list.
    fn brelse(&mut self, index: usize) {
        let buf = &mut self.inner[index];
        buf.refcnt -= 1;

        if buf.refcnt == 0 && !ptr::eq(self.head, buf) {
            if ptr::eq(self.tail, buf) && !buf.prev.is_null() {
                self.tail = buf.prev;
            }

            unsafe {
                buf.next.as_mut().map(|buf_next| buf_next.prev = buf.prev);
                buf.prev.as_mut().map(|buf_prev| buf_prev.next = buf.next);
            }

            buf.prev = ptr::null_mut();
            buf.next = self.head;
            unsafe {
                self.head.as_mut().map(|old_head| old_head.prev = buf);
            }
            self.head = buf;
        }
    }
}

struct BufInfo {
    index: usize,
    dev: u32,
    blockno: u32,
    refcnt: usize,
    prev: *mut BufInfo,
    next: *mut BufInfo,
}

impl BufInfo {
    const fn new(index: usize) -> Self {
        Self {
            index,
            dev: 0,
            blockno: 0,
            refcnt: 0,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}
