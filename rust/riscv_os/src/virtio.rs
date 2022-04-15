/// driver for qemu's virtio disk device.
/// uses qemu's mmio interface to virtio.
/// qemu presents a "legacy" virtio interface.
use core::{
    mem, ptr,
    sync::atomic::{fence, Ordering},
};

use crate::{
    bio::{GuardBuf, BSIZE},
    cpu::CPU_TABLE,
    param::{PAGESIZE, VIRTIO0},
    println,
    process::PROCESS_TABLE,
    spinlock::SpinLock,
};
use array_macro::array;

pub static DISK: SpinLock<Disk> = SpinLock::new(Disk::new());

#[repr(C)]
struct Desc {
    addr: usize,
    len: u32,
    flags: u16,
    next: u16,
}

impl Desc {
    const fn new() -> Self {
        Self {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

#[repr(C)]
struct Used {
    flags: u16,
    idx: u16,
    ring: [UsedElem; NUM as usize],
}

impl Used {
    const fn new() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: array![_ => UsedElem::new(); NUM as usize],
        }
    }
}

#[repr(C)]
struct UsedElem {
    id: u32,
    len: u32,
}

impl UsedElem {
    const fn new() -> Self {
        Self { id: 0, len: 0 }
    }
}

#[repr(C)]
struct Info {
    buf_chan: Option<usize>,
    disk: bool,
    status: u8,
}

impl Info {
    const fn new() -> Self {
        Self {
            buf_chan: None,
            disk: false,
            status: 0,
        }
    }
}

#[repr(C)]
struct BlkReq {
    typed: u32,
    reserved: u32,
    sector: usize,
}

impl BlkReq {
    const fn new() -> Self {
        Self {
            typed: 0,
            reserved: 0,
            sector: 0,
        }
    }
}

const AVAILSIZE: usize =
    (PAGESIZE - NUM as usize * core::mem::size_of::<Desc>()) / core::mem::size_of::<u16>();

#[repr(C)]
#[repr(align(4096))]
pub struct Disk {
    // start pages
    // that is devided three regions (decriptors, avail, and used).
    // https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.pdf
    desc: [Desc; NUM as usize],
    avail: [u16; AVAILSIZE],
    used: [Used; NUM as usize],

    free: [bool; NUM as usize], // is a descriptor free?
    used_idx: u32,
    info: [Info; NUM as usize],
    ops: [BlkReq; NUM as usize],
}

impl Disk {
    const fn new() -> Self {
        Self {
            desc: array![_ => Desc::new(); NUM as usize],
            avail: [0; AVAILSIZE],
            used: array![_ => Used::new(); NUM as usize],
            free: [false; NUM as usize],
            used_idx: 0,
            info: array![_ => Info::new(); NUM as usize],
            ops: array![_ => BlkReq::new(); NUM as usize],
        }
    }

    pub unsafe fn init(&mut self) {
        if read(VIRTIO_MMIO_MAGIC_VALUE) != 0x74726976
            || read(VIRTIO_MMIO_VERSION) != 1
            || read(VIRTIO_MMIO_DEVICE_ID) != 2
            || read(VIRTIO_MMIO_VENDOR_ID) != 0x554d4551
        {
            panic!("could not find virtio disk");
        }

        let mut status: u32 = 0;
        status |= VIRTIO_CONFIG_S_ACKNOWLEDGE;
        write(VIRTIO_MMIO_STATUS, status);
        status |= VIRTIO_CONFIG_S_DRIVER;
        write(VIRTIO_MMIO_STATUS, status);

        // negotiate features
        let mut features: u32 = read(VIRTIO_MMIO_DEVICE_FEATURES);
        features &= !(1u32 << VIRTIO_BLK_F_RO);
        features &= !(1u32 << VIRTIO_BLK_F_SCSI);
        features &= !(1u32 << VIRTIO_BLK_F_CONFIG_WCE);
        features &= !(1u32 << VIRTIO_BLK_F_MQ);
        features &= !(1u32 << VIRTIO_F_ANY_LAYOUT);
        features &= !(1u32 << VIRTIO_RING_F_EVENT_IDX);
        features &= !(1u32 << VIRTIO_RING_F_INDIRECT_DESC);
        write(VIRTIO_MMIO_DRIVER_FEATURES, features);

        // tell device that feature negotiation is complete.
        status |= VIRTIO_CONFIG_S_FEATURES_OK;
        write(VIRTIO_MMIO_STATUS, status);

        // tell device we're complete ready.
        status |= VIRTIO_CONFIG_S_DRIVER_OK;
        write(VIRTIO_MMIO_STATUS, status);

        write(VIRTIO_MMIO_GUEST_PAGE_SIZE, PAGESIZE as u32);

        // initialize queue 0.
        write(VIRTIO_MMIO_QUEUE_SEL, 0);
        let max: u32 = read(VIRTIO_MMIO_QUEUE_NUM_MAX);
        if max == 0 {
            panic!("virtio disk has no queue 0");
        } else if max < NUM {
            panic!("virtio disk max queue too short");
        }
        write(VIRTIO_MMIO_QUEUE_NUM, NUM);

        let pfn: usize = (self as *const Disk as usize) >> 12;
        println!("DISK pfn: {:#x}", pfn);
        write(VIRTIO_MMIO_QUEUE_PFN, u32::try_from(pfn).unwrap());

        // all NUM descriptors start out unused.
        self.free.iter_mut().for_each(|v| *v = true);

        println!("virtio: init virtio driver done");
    }

    pub fn intr(&mut self) {
        unsafe {
            write(
                VIRTIO_MMIO_INTERRUPT_ACK,
                read(VIRTIO_MMIO_INTERRUPT_STATUS) & 0x3,
            )
        };

        fence(Ordering::SeqCst);

        while self.used_idx != self.used[0].idx as u32 {
            fence(Ordering::SeqCst);

            let id = self.used[0].ring[(self.used_idx % NUM) as usize].id as usize;

            if self.info[id].status != 0 {
                panic!("virtio_intr: status");
            }

            let buf = self.info[id]
                .buf_chan
                .clone()
                .expect("virtio: intr not found buffer channel");
            unsafe {
                PROCESS_TABLE.wakeup(buf);
            }
            self.info[id].disk = false;
            self.used_idx += 1;
        }
    }

    fn alloc_desc(&mut self) -> Option<usize> {
        for i in 0..(NUM as usize) {
            if self.free[i] {
                self.free[i] = false;
                return Some(i);
            }
        }

        None
    }

    fn free_desc(&mut self, i: usize) {
        // TODO: use assert expr
        if i >= (NUM as usize) {
            panic!("free_desc: out of range: {}", i);
        }
        if self.free[i] {
            panic!("free_desc: already free: {}", i);
        }

        self.desc[i].addr = 0;
        self.desc[i].len = 0;
        self.desc[i].flags = 0;
        self.desc[i].next = 0;
        self.free[i] = true;

        unsafe {
            PROCESS_TABLE.wakeup(&self.free[0] as *const bool as usize);
        }
    }

    fn alloc3_desc(&mut self, idx: &mut [usize; 3]) -> bool {
        for i in 0..3 {
            match self.alloc_desc() {
                Some(desc) => {
                    idx[i] = desc;
                }
                None => {
                    for j in 0..i {
                        self.free_desc(j);
                    }
                    return false;
                }
            }
        }

        true
    }

    fn free_chain(&mut self, i: usize) {
        let mut i = i;
        // print!("free_chain: free...");
        loop {
            let should = (self.desc[i].flags & VRING_DESC_F_NEXT) != 0;
            let next = self.desc[i].next;
            // print!(" {}", i);
            self.free_desc(i);
            if !should {
                break;
            }
            i = next as usize;
        }
        // println!();
    }
}

impl SpinLock<Disk> {
    /// block operations use three descriptors:
    /// one for type/reserved/sector
    /// one for the data
    /// one for a 1-byte status result
    pub fn rw(&self, buf: &mut GuardBuf, writing: bool) {
        let mut locked = self.lock();

        // allocate three descriptors
        let mut idx = [0usize; 3];
        loop {
            if locked.alloc3_desc(&mut idx) {
                break;
            }
            unsafe {
                CPU_TABLE
                    .my_proc()
                    .sleep(&locked.free[0] as *const _ as usize, locked);
            }
            locked = self.lock();
        }

        // format the three descriptors
        let buf0 = &mut locked.ops[idx[0]];
        buf0.typed = if writing {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };
        buf0.reserved = 0;
        buf0.sector = (buf.blockno as usize * (BSIZE / 512)) as usize;

        // buf0 (type/reserved/sector)
        locked.desc[idx[0]].addr = buf0 as *mut _ as usize;
        locked.desc[idx[0]].len = mem::size_of::<BlkReq>().try_into().unwrap();
        locked.desc[idx[0]].flags = VRING_DESC_F_NEXT;
        locked.desc[idx[0]].next = idx[1].try_into().unwrap();

        // data
        let buf_ptr = buf.data_ptr_mut();
        locked.desc[idx[1]].addr = buf_ptr as usize;
        locked.desc[idx[1]].len = BSIZE.try_into().unwrap();
        locked.desc[idx[1]].flags = if writing { 0 } else { VRING_DESC_F_WRITE };
        locked.desc[idx[1]].flags |= VRING_DESC_F_NEXT;
        locked.desc[idx[1]].next = idx[2].try_into().unwrap();

        // status result
        let status_addr = &mut locked.info[idx[0]].status as *mut _ as usize;
        locked.info[idx[0]].status = 0xff; // device writes 0 on success
        locked.desc[idx[2]].addr = status_addr;
        locked.desc[idx[2]].len = 1;
        locked.desc[idx[2]].flags = VRING_DESC_F_WRITE;
        locked.desc[idx[2]].next = 0;

        // record struct buf for intr()
        locked.info[idx[0]].disk = true;
        locked.info[idx[0]].buf_chan = Some(buf_ptr as usize);

        // tell the device the first index in our chain of descriptors.
        let avail_idx = 2 + locked.avail[1] as usize % (NUM as usize);
        locked.avail[avail_idx] = idx[0].try_into().unwrap();

        fence(Ordering::SeqCst);

        // tell the device another avail ring entry is available
        locked.avail[1] += 1;

        fence(Ordering::SeqCst);

        unsafe {
            write(VIRTIO_MMIO_QUEUE_NOTIFY, 0);
        }

        // wait for intr() to say request has finised
        while locked.info[idx[0]].disk {
            unsafe {
                CPU_TABLE.my_proc().sleep(buf_ptr as usize, locked);
            }
            locked = self.lock();
        }
        // tidy up
        let res = locked.info[idx[0]].buf_chan.take();
        assert_eq!(res.unwrap(), buf_ptr as usize);
        locked.free_chain(idx[0]);

        drop(locked);
    }
}

const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
const VIRTIO_MMIO_VERSION: usize = 0x004;
const VIRTIO_MMIO_DEVICE_ID: usize = 0x008; // device type; 1 is net, 2 is disk
const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x020;
const VIRTIO_MMIO_GUEST_PAGE_SIZE: usize = 0x028; // page size for PFN, write-only
const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
const VIRTIO_MMIO_QUEUE_ALIGN: usize = 0x03c;
const VIRTIO_MMIO_QUEUE_PFN: usize = 0x040;
const VIRTIO_MMIO_QUEUE_READY: usize = 0x044;
const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
const VIRTIO_MMIO_INTERRUPT_STATUS: usize = 0x060;
const VIRTIO_MMIO_INTERRUPT_ACK: usize = 0x064;
const VIRTIO_MMIO_STATUS: usize = 0x070; // read/write

const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
const VIRTIO_CONFIG_S_DRIVER: u32 = 2;
const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 4;
const VIRTIO_CONFIG_S_FEATURES_OK: u32 = 8;

const VIRTIO_BLK_F_RO: u8 = 5;
const VIRTIO_BLK_F_SCSI: u8 = 7;
const VIRTIO_BLK_F_CONFIG_WCE: u8 = 11;
const VIRTIO_BLK_F_MQ: u8 = 12;
const VIRTIO_F_ANY_LAYOUT: u8 = 27;
const VIRTIO_RING_F_INDIRECT_DESC: u8 = 28;
const VIRTIO_RING_F_EVENT_IDX: u8 = 29;

const VRING_DESC_F_NEXT: u16 = 1; // chained with another descriptor
const VRING_DESC_F_WRITE: u16 = 2; // device writes (vs read)

const VIRTIO_BLK_T_IN: u32 = 0; // read the disk
const VIRTIO_BLK_T_OUT: u32 = 1; // write the disk

const NUM: u32 = 8; // this many virtio descriptors. must be a power of two.

#[inline]
unsafe fn read(offset: usize) -> u32 {
    let src = (VIRTIO0 + offset) as *const u32;
    ptr::read_volatile(src)
}

#[inline]
unsafe fn write(offset: usize, v: u32) {
    let dst = (VIRTIO0 + offset) as *mut u32;
    ptr::write_volatile(dst, v);
}

pub mod tests {

    use super::*;

    pub fn tests() -> &'static [(&'static str, fn())] {
        &[
            ("memory layout", test_memory_layout),
            ("read write", test_rw),
        ]
    }

    pub fn test_memory_layout() {
        let disk = DISK.lock();
        assert_eq!(&disk.desc as *const _ as usize % PAGESIZE, 0);
        assert_eq!(&disk.used as *const _ as usize % PAGESIZE, 0);
        assert_eq!(
            &disk.used as *const _ as usize - &disk.desc as *const _ as usize,
            PAGESIZE
        );
    }

    pub fn test_rw() {
        // let buf = BCACHE.bread(ROOTDEV, 1);
        // let buf_ptr = buf.data_ptr();
    }
}
