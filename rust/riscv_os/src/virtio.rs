/// driver for qemu's virtio disk device.
/// uses qemu's mmio interface to virtio.
/// qemu presents a "legacy" virtio interface.
use core::{
    ptr,
    sync::atomic::{fence, Ordering},
};

use crate::{
    param::{PAGESIZE, VIRTIO0},
    println,
    process::PROCESS_TABLE,
    spinlock::SpinLock,
};
use array_macro::array;

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

pub static DISK: SpinLock<Disk> = SpinLock::new(Disk::new());

#[repr(align(4096))]
pub struct Disk {
    align1: PageAlign,
    desc: Desc,
    avail: Avail,
    used: Used,
    free: [bool; NUM as usize], // is a descriptor free?
    used_idx: u32,
    info: [Info; NUM as usize],
}

impl Disk {
    const fn new() -> Self {
        Self {
            align1: PageAlign::new(),
            desc: Desc::new(),
            avail: Avail::new(),
            used: Used::new(),
            free: [false; NUM as usize],
            used_idx: 0,
            info: array![_ => Info::new(); NUM as usize],
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
        let mut features = read(VIRTIO_MMIO_DEVICE_FEATURES);
        features &= !(1 << VIRTIO_BLK_F_RO);
        features &= !(1 << VIRTIO_BLK_F_SCSI);
        features &= !(1 << VIRTIO_BLK_F_CONFIG_WCE);
        features &= !(1 << VIRTIO_BLK_F_MQ);
        features &= !(1 << VIRTIO_F_ANY_LAYOUT);
        features &= !(1 << VIRTIO_RING_F_EVENT_IDX);
        features &= !(1 << VIRTIO_RING_F_INDIRECT_DESC);
        write(VIRTIO_MMIO_DEVICE_FEATURES, features);

        // tell device that feature negotiation is complete.
        status |= VIRTIO_CONFIG_S_FEATURE_OK;
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

        let pfn: usize = self as *const Disk as usize >> 12;
        write(VIRTIO_MMIO_QUEUE_PFN, u32::try_from(pfn).unwrap());

        // all NUM descriptors start out unused.
        self.free.iter_mut().for_each(|v| *v = true);

        println!("virtio: init virtio driver done");
    }

    pub fn intr(&mut self) {
        fence(Ordering::SeqCst);

        while self.used_idx != self.used.idx as u32 {
            fence(Ordering::SeqCst);

            let id = self.used.ring[(self.used_idx % NUM) as usize].id as usize;

            if self.info[id].status != 0 {
                panic!("virtio_intr: status");
            }

            let buf = self.info[id].buf_chan.clone().expect("");
            unsafe {
                PROCESS_TABLE.wakeup(buf);
            }
            self.info[id].disk = false;
            self.used_idx += 1;
        }

        println!("virtio: intr done");
    }
}

#[repr(align(4096))]
struct PageAlign();

impl PageAlign {
    const fn new() -> Self {
        Self()
    }
}

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
struct Avail {
    flags: u16,
    idx: u16,
    ring: [u16; NUM as usize],
    unused: u16,
}

impl Avail {
    const fn new() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: array![_ => 0_u16; NUM as usize],
            unused: 0,
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

const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x000;
const VIRTIO_MMIO_VERSION: usize = 0x004;
const VIRTIO_MMIO_DEVICE_ID: usize = 0x008; // device type; 1 is net, 2 is disk
const VIRTIO_MMIO_VENDOR_ID: usize = 0x00c;
const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x010;
const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x010;
const VIRTIO_MMIO_GUEST_PAGE_SIZE: usize = 0x028; // page size for PFN, write-only
const VIRTIO_MMIO_QUEUE_SEL: usize = 0x030;
const VIRTIO_MMIO_QUEUE_NUM_MAX: usize = 0x034;
const VIRTIO_MMIO_QUEUE_NUM: usize = 0x038;
const VIRTIO_MMIO_QUEUE_ALIGN: usize = 0x03c;
const VIRTIO_MMIO_QUEUE_PFN: usize = 0x040;
const VIRTIO_MMIO_QUEUE_READY: usize = 0x044;
const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x050;
const VIRTIO_MMIO_STATUS: usize = 0x070; // read/write

const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
const VIRTIO_CONFIG_S_DRIVER: u32 = 2;
const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 4;
const VIRTIO_CONFIG_S_FEATURE_OK: u32 = 8;

const VIRTIO_BLK_F_RO: u32 = 5;
const VIRTIO_BLK_F_SCSI: u32 = 7;
const VIRTIO_BLK_F_CONFIG_WCE: u32 = 11;
const VIRTIO_BLK_F_MQ: u32 = 12;
const VIRTIO_F_ANY_LAYOUT: u32 = 27;
const VIRTIO_RING_F_INDIRECT_DESC: u32 = 28;
const VIRTIO_RING_F_EVENT_IDX: u32 = 29;

const NUM: u32 = 8; // this many virtio descriptors. must be a power of two.
