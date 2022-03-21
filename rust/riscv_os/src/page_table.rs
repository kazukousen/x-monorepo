use crate::kalloc::kalloc;
use crate::param::PAGESIZE;
use bitflags::bitflags;
use core::ops::{Index, IndexMut};

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
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn empty() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        Self {
            entries: [EMPTY; 512],
        }
    }

    pub fn map_pages(
        &mut self,
        va: usize,
        pa: usize,
        size: usize,
        perm: PteFlag,
    ) -> Result<(), &'static str> {
        let mut va_start = align_down(va, PAGESIZE);
        let mut va_end = align_down(va + size - 1, PAGESIZE);

        while va_start != va_end {
            match self.walk(va_start) {
                Some(pte) => {
                    if pte.is_unused() {
                        pte.set_addr(as_pte_addr(pa), perm);
                    } else {
                        return Err("map_pages: remap");
                    }
                }
                None => {
                    return Err("map_pages: not enough memory for new page table");
                }
            }

            va_start += PAGESIZE;
            va_end += PAGESIZE;
        }

        Ok(())
    }

    fn walk(&mut self, va: usize) -> Option<&mut PageTableEntry> {
        let mut page_table = self as *mut PageTable;

        for level in (1..=2).rev() {
            let mut pte = unsafe { &mut page_table.as_mut().unwrap()[get_index(va, level)] };

            if pte.is_unused() {
                let ptr = kalloc();
                if ptr == 0 as *mut u8 {
                    return None;
                }
                pte.set_addr(as_pte_addr(ptr as usize), PteFlag::Valid);
            }

            page_table = pte.as_page_table();
        }

        unsafe { Some(&mut page_table.as_mut().unwrap()[get_index(va, 0)]) }
    }
}

enum PageTableLevel {
    Zero,
    One,
    Two,
}

fn get_index(va: usize, level: usize) -> PageTableIndex {
    PageTableIndex((va >> (12 + level * 9) & 0x1FF) as u16)
}

fn as_pte_addr(pa: usize) -> usize {
    pa >> 12 << 10
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index.0)]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index.0)]
    }
}

/// A 9-bits index for page table.
pub struct PageTableIndex(u16);

#[repr(C)]
pub struct PageTableEntry {
    data: usize,
}

impl PageTableEntry {
    #[inline]
    pub const fn new() -> Self {
        Self { data: 0 }
    }

    #[inline]
    pub fn is_unused(&self) -> bool {
        self.data == 0
    }

    pub fn set_addr(&mut self, addr: usize, perm: PteFlag) {
        self.data = addr | perm.bits();
    }

    #[inline]
    fn addr(&self) -> usize {
        self.data >> 10 << 12
    }

    #[inline]
    fn as_page_table(&self) -> *mut PageTable {
        (self.data >> 10 << 12) as *mut PageTable
    }
}

const fn align_down(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    addr & !(align - 1)
}

const fn align_up(addr: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}
