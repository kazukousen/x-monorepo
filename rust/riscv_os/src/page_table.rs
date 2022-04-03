use crate::param::{PAGESIZE, TRAMPOLINE, TRAPFRAME};
use crate::println;
use alloc::boxed::Box;
use bitflags::bitflags;
use core::alloc::AllocError;
use core::cmp::min;
use core::ops::{Index, IndexMut};
use core::ptr;

bitflags! {
    pub struct PteFlag: usize {
        const VALID = 1 << 0;
        const READ = 1 << 1;
        const WRITE = 1 << 2;
        const EXEC = 1 << 3;
        const USER = 1 << 4;
        const GLOB = 1 << 5;
        const ACCES = 1 << 6;
        const DIRTY = 1 << 7;
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
pub struct SinglePage {
    data: [u8; PAGESIZE],
}

impl Page for SinglePage {}

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
            fn trampoline();
        }
        let mut pt = unsafe { Box::<Self>::try_new_zeroed().ok()?.assume_init() };

        pt.map_pages(
            TRAMPOLINE,
            trampoline as usize,
            PAGESIZE,
            PteFlag::READ | PteFlag::EXEC,
        )
        .ok()?;

        pt.map_pages(
            TRAPFRAME,
            trapframe,
            PAGESIZE,
            PteFlag::READ | PteFlag::WRITE,
        )
        .ok()?;

        Some(pt)
    }

    /// Load the user initcode into address 0 of pagetable,
    /// for the very first process.
    /// sz must be less than a page.
    pub fn uvm_init(&mut self, code: &[u8]) -> Result<(), &'static str> {
        if code.len() >= PAGESIZE {
            return Err("uvm_init: more than a page");
        }

        let mem = unsafe { SinglePage::new_zeroed().or(Err("uvm_init: insufficient memory"))? };
        self.map_pages(
            0,
            mem as usize,
            PAGESIZE,
            PteFlag::READ | PteFlag::WRITE | PteFlag::EXEC | PteFlag::USER,
        )?;

        // copy the code
        unsafe {
            ptr::copy_nonoverlapping(code.as_ptr(), mem, code.len());
        }

        Ok(())
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
            match self.walk_alloc(va) {
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

    pub fn walk_addr(&self, va: usize) -> Result<usize, &'static str> {
        match self.walk(va) {
            Some(pte) => {
                if !pte.is_valid() {
                    Err("walk_addr: pte is not valid")
                } else if !pte.is_user() {
                    Err("walk_addr: pte is not user")
                } else {
                    Ok(pte.as_phys_addr())
                }
            }
            None => Err("walk_addr: va is not mapped"),
        }
    }

    fn walk(&self, va: usize) -> Option<&PageTableEntry> {
        let mut page_table = self as *const PageTable;

        for level in (1..=2).rev() {
            let pte = unsafe { &page_table.as_ref().unwrap()[get_index(va, level)] };

            if !pte.is_valid() {
                return None;
            }

            page_table = pte.as_page_table();
        }

        unsafe { Some(&page_table.as_ref().unwrap()[get_index(va, 0)]) }
    }

    fn walk_alloc(&mut self, va: usize) -> Option<&mut PageTableEntry> {
        let mut page_table = self as *mut PageTable;

        for level in (1..=2).rev() {
            let pte = unsafe { &mut page_table.as_mut().unwrap()[get_index(va, level)] };

            if !pte.is_valid() {
                // The raw page_table pointer is leaked but kept in the page table entry that can calculate later.
                let page_table_ptr = unsafe { PageTable::new_zeroed().ok()? };

                pte.set_addr(as_pte_addr(page_table_ptr as usize), PteFlag::VALID);
            }

            page_table = pte.as_page_table();
        }

        unsafe { Some(&mut page_table.as_mut().unwrap()[get_index(va, 0)]) }
    }

    /// Copy a null-terminated string from user to kernel.
    /// Copy bytes to dst from virtual address srcva in a given page table,
    /// until a '\0'.
    pub fn copy_in_str(&self, dst: &mut [u8], srcva: usize) -> Result<(), &'static str> {
        let mut i = 0;
        let mut va = srcva;

        while i < dst.len() {
            let va_base = align_down(va, PAGESIZE);
            let distance = va - va_base;
            let mut va_ptr = va as *const u8;
            let mut pa_ptr =
                unsafe { (self.walk_addr(va_base)? as *const u8).offset(distance as isize) };

            let mut count = min(PAGESIZE - distance, dst.len() - 1);
            while count > 0 {
                unsafe {
                    dst[i] = ptr::read(pa_ptr);
                    if dst[i] == 0 {
                        return Ok(());
                    }
                    va_ptr = va_ptr.add(1);
                    pa_ptr = pa_ptr.add(1);
                    i += 1;
                    count -= 1;
                }
            }

            va = va_base + PAGESIZE;
        }

        Err("copy_in_str: dst not enough space")
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
        (self.data & PteFlag::VALID.bits()) > 0
    }

    #[inline]
    pub fn is_user(&self) -> bool {
        (self.data & PteFlag::USER.bits()) > 0
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        (self.data & (PteFlag::READ | PteFlag::WRITE | PteFlag::EXEC).bits()) > 0
    }

    pub fn set_addr(&mut self, addr: usize, perm: PteFlag) {
        self.data = addr | (perm | PteFlag::VALID).bits();
    }

    #[inline]
    fn as_page_table(&self) -> *mut PageTable {
        // Physical Page Number (44 bit) + Offset (12 bit)
        (self.data >> 10 << 12) as *mut PageTable
    }

    #[inline]
    fn as_phys_addr(&self) -> usize {
        // Physical Page Number (44 bit) + Offset (12 bit)
        (self.data >> 10 << 12)
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
