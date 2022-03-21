use bitflags::bitflags;

bitflags! {
    pub struct PteFlag: usize {
        const Valid = 1 << 0;
        const Read = 1 << 1;
        const Write = 1 << 2;
        const Exec = 1 << 3;
        const User = 1 << 4;
        const Glob = 1 << 5;
        const Acces = 1 << 6;
        const Dirty = 1 << 7;
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512]
}

impl PageTable {
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        Self {
            entries: [EMPTY; 512],
        }
    }

    fn walk(&self, va: usize) {
        let mut pgt = self as *mut PageTable;
    }
}

#[repr(C)]
pub struct PageTableEntry {
    data: usize,
}

impl PageTableEntry {

    #[inline]
    pub const fn new() -> Self {
        Self {
            data: 0,
        }
    }

    #[inline]
    pub fn is_unused(&self) -> bool {
        self.data == 0
    }

    pub fn set_addr(&mut self, addr: usize, perm: PteFlag) {
        self.data = addr | perm.bits();
    }
}
