use crate::param::{PAGESIZE, TRAMPOLINE, TRAPFRAME};
use crate::println;
use alloc::boxed::Box;
use bitflags::bitflags;
use core::alloc::AllocError;
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

pub trait Page: Sized {
    unsafe fn new_zeroed() -> Result<*mut u8, AllocError> {
        let page = Box::<Self>::try_new_zeroed()?.assume_init();
        Ok(Box::into_raw(page) as *mut u8)
    }

    unsafe fn drop(raw: *mut u8) {
        drop(Box::from_raw(raw as *mut Self))
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl Page for PageTable {}

impl PageTable {
    pub const fn empty() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        Self {
            entries: [EMPTY; 512],
        }
    }

    // Allocate a new user page table.
    pub fn alloc_user_page_table(trapframe: usize) -> Option<Box<Self>> {
        extern "C" {
            fn _trampoline();
        }
        let trampoline = _trampoline as usize;

        let mut pt = unsafe { Box::<Self>::try_new_zeroed().ok()?.assume_init() };

        pt.map_pages(
            TRAMPOLINE,
            trampoline,
            PAGESIZE,
            PteFlag::Read | PteFlag::Exec,
        )
        .ok()?;

        pt.map_pages(
            TRAPFRAME,
            trapframe,
            PAGESIZE,
            PteFlag::Read | PteFlag::Write,
        )
        .ok()?;

        Some(pt)
    }

    pub fn as_satp(&self) -> usize {
        (8 << 60) | ((self as *const PageTable as usize) >> 12)
    }

    pub fn map_pages(
        &mut self,
        va: usize,
        pa: usize,
        size: usize,
        perm: PteFlag,
    ) -> Result<(), &'static str> {
        let va_start = align_down(va, PAGESIZE);
        let va_end = align_up(va + size, PAGESIZE);

        let mut pa = pa;

        for va in (va_start..va_end).step_by(PAGESIZE) {
            // println!("va_start={:#x}, va_end={:#x}, pa={:#x}, size={:#x}", va, va_end, pa, size);
            match self.walk(va) {
                Some(pte) => {
                    if pte.is_valid() {
                        return Err("map_pages: remap");
                    } else {
                        pte.set_addr(as_pte_addr(pa), perm);
                    }
                }
                None => {
                    return Err("map_pages: not enough memory for new page table");
                }
            }

            pa += PAGESIZE;
        }

        Ok(())
    }

    fn walk(&mut self, va: usize) -> Option<&mut PageTableEntry> {
        let mut page_table = self as *mut PageTable;

        for level in (1..=2).rev() {
            let pte = unsafe { &mut page_table.as_mut().unwrap()[get_index(va, level)] };

            if !pte.is_valid() {
                // The raw page_table pointer is leaked but kept in the page table entry that can calculate later.
                let page_table_ptr = unsafe { PageTable::new_zeroed().ok()? };

                pte.set_addr(as_pte_addr(page_table_ptr as usize), PteFlag::Valid);
            }

            page_table = pte.as_page_table();
        }

        unsafe { Some(&mut page_table.as_mut().unwrap()[get_index(va, 0)]) }
    }
}

impl Drop for PageTable {
    fn drop(&mut self) {
        self.entries.iter_mut().for_each(|e| e.free());
    }
}

fn get_index(va: usize, level: usize) -> PageTableIndex {
    PageTableIndex(((va >> (12 + level * 9)) & 0x1FF) as u16)
}

fn as_pte_addr(pa: usize) -> usize {
    (pa >> 12) << 10
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

#[derive(Debug)]
#[repr(C)]
pub struct PageTableEntry {
    data: usize, // Physical Page Number (44 bit) + Flags (10 bit)
}

impl PageTableEntry {
    #[inline]
    pub const fn new() -> Self {
        Self { data: 0 }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        (self.data & PteFlag::Valid.bits()) > 0
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        (self.data & (PteFlag::Read | PteFlag::Write | PteFlag::Exec).bits()) > 0
    }

    pub fn set_addr(&mut self, addr: usize, perm: PteFlag) {
        self.data = addr | (perm | PteFlag::Valid).bits();
    }

    #[inline]
    fn as_page_table(&self) -> *mut PageTable {
        // Physical Page Number (44 bit) + Offset (12 bit)
        (self.data >> 10 << 12) as *mut PageTable
    }

    fn free(&mut self) {
        if self.is_valid() {
            if self.is_leaf() {
                panic!("freeing a PTE leaf")
            }
            drop(unsafe { Box::from_raw(self.as_page_table()) })
        }
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
