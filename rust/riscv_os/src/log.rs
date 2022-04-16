use core::ptr;

use crate::{bio::BCACHE, fs::SuperBlock, param::LOGSIZE, println, spinlock::SpinLock};

#[repr(C)]
struct LogHeader {
    n: u32,
    blocks: [u32; LOGSIZE],
}

impl LogHeader {
    const fn new() -> Self {
        Self {
            n: 0,
            blocks: [0; LOGSIZE],
        }
    }
}

pub struct Log {
    start: u32,
    size: u32,
    outstanding: usize, // how many FS sys calls are executing.
    committing: bool,   // in commit(), please wait.
    dev: u32,
    header: LogHeader,
}

pub static LOG: SpinLock<Log> = SpinLock::new(Log::new());

impl Log {
    const fn new() -> Self {
        Self {
            start: 0,
            size: 0,
            outstanding: 0,
            committing: true,
            dev: 0,
            header: LogHeader::new(),
        }
    }

    pub fn init(&mut self, dev: u32, sb: &SuperBlock) {
        self.start = sb.logstart;
        self.size = sb.nlog;
        self.dev = dev;
        self.recover_from_log();
        println!("log init done");
    }

    fn recover_from_log(&mut self) {
        self.read_head();
        self.install_trans(true);
        self.header.n = 0;
        self.write_head();
    }

    fn read_head(&mut self) {
        let buf = BCACHE.bread(self.dev, self.start);

        unsafe {
            ptr::copy_nonoverlapping(buf.data_ptr() as *const LogHeader, &mut self.header, 1);
        }
        // TODO: brelse?
        drop(buf);
    }

    fn install_trans(&mut self, recovering: bool) {
        for tail in 0..self.header.n {
            let logbuf = BCACHE.bread(self.dev, self.start + tail + 1); // read log block
            let mut dstbuf = BCACHE.bread(self.dev, self.header.blocks[tail as usize]); // read dst
            unsafe {
                ptr::copy_nonoverlapping(logbuf.data_ptr(), dstbuf.data_ptr_mut(), 1);
            }
            dstbuf.bwrite();
            if !recovering {
                unsafe {
                    dstbuf.bunpin();
                }
            }
            // TODO: brelse?
            drop(logbuf);
            drop(dstbuf);
        }
    }

    fn write_head(&mut self) {
        let mut buf = BCACHE.bread(self.dev, self.start);

        unsafe {
            ptr::copy_nonoverlapping(&self.header, buf.data_ptr_mut() as *mut LogHeader, 1);
        }
        buf.bwrite();
        // TODO: brelse?
        drop(buf);
    }
}
