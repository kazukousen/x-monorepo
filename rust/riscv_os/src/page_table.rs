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
    unsafe fn alloc_into_raw() -> Result<*mut Self, AllocError> {
        let page = Box::<Self>::try_new_zeroed()?.assume_init();
        Ok(Box::into_raw(page))
    }

    unsafe fn free_from_raw(raw: *mut Self) {
        drop(Box::from_raw(raw))
    }
}

#[repr(C, align(4096))]
pub struct SinglePage {
    data: [u8; PAGESIZE],
}

impl Page for SinglePage {}

#[repr(C, align(4096))]
pub struct QuadPage {
    data: [u8; PAGESIZE * 4],
}

impl Page for QuadPage {}

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

    pub fn as_satp(&self) -> usize {
        (8 << 60) | ((self as *const PageTable as usize) >> 12)
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

    /// Unmap process's pages.
    pub fn unmap_user_page_table(&mut self, sz: usize) {
        self.unmap_pages(TRAMPOLINE, 1, false)
            .expect("cannot unmap trampoline");
        self.unmap_pages(TRAPFRAME, 1, false)
            .expect("cannot unmap trampframe");
        if sz > 0 {
            self.unmap_pages(0, align_up(sz, PAGESIZE) / PAGESIZE, true)
                .expect("cannot unmap process");
        }
    }

    /// Allocate PTEs and physical memory to grow process from oldsz to newsz, which need not to be
    /// aligned. returns new size or an error.
    pub fn uvm_alloc(&mut self, oldsz: usize, newsz: usize) -> Result<usize, &'static str> {
        if newsz <= oldsz {
            return Ok(oldsz);
        }

        let oldsz = align_up(oldsz, PAGESIZE);
        for va in (oldsz..newsz).step_by(PAGESIZE) {
            let mem = unsafe {
                match SinglePage::alloc_into_raw() {
                    Ok(mem) => mem,
                    Err(_) => {
                        self.uvm_dealloc(oldsz, newsz)?;
                        return Err("uvm_alloc: insufficient memory");
                    }
                }
            };
            match self.map_pages(
                va,
                mem as usize,
                PAGESIZE,
                PteFlag::READ | PteFlag::WRITE | PteFlag::EXEC | PteFlag::USER,
            ) {
                Err(msg) => {
                    unsafe { SinglePage::free_from_raw(mem) };
                    self.uvm_dealloc(oldsz, newsz)?;
                    return Err(msg);
                }
                Ok(_) => {
                    // ok, the mem pointer is leaked, but stored in the page table at virt address `va`.
                }
            };
        }

        Ok(newsz)
    }

    fn uvm_dealloc(&mut self, mut oldsz: usize, mut newsz: usize) -> Result<usize, &'static str> {
        if newsz >= oldsz {
            return Ok(oldsz);
        }

        oldsz = align_up(oldsz, PAGESIZE);
        newsz = align_up(newsz, PAGESIZE);
        if newsz < oldsz {
            self.unmap_pages(newsz, (oldsz - newsz) / PAGESIZE, true)?;
        }

        Ok(newsz)
    }

    pub fn uvm_clear(&mut self, va: usize) {
        let pte = self.walk_mut(va).expect("uvm_clear");
        pte.data &= !PteFlag::USER.bits();
    }

    /// Load the user initcode into address 0 of pagetable,
    /// for the very first process.
    /// sz must be less than a page.
    pub fn uvm_init(&mut self, code: &[u8]) -> Result<(), &'static str> {
        if code.len() >= PAGESIZE {
            return Err("uvm_init: more than a page");
        }

        let mem = unsafe { SinglePage::alloc_into_raw().or(Err("uvm_init: insufficient memory"))? };
        self.map_pages(
            0,
            mem as usize,
            PAGESIZE,
            PteFlag::READ | PteFlag::WRITE | PteFlag::EXEC | PteFlag::USER,
        )?;

        // copy the code
        unsafe {
            ptr::copy_nonoverlapping(code.as_ptr(), mem as *mut u8, code.len());
        }

        Ok(())
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

    fn unmap_pages(
        &mut self,
        va_start: usize,
        n: usize,
        freeing: bool,
    ) -> Result<(), &'static str> {
        if va_start % PAGESIZE != 0 {
            panic!("unmap_pages: not aligned");
        }

        for va in (va_start..(va_start + n * PAGESIZE)).step_by(PAGESIZE) {
            match self.walk_mut(va) {
                Some(pte) => {
                    if !pte.is_valid() {
                        return Err("not mapped");
                    }
                    if !pte.is_leaf() {
                        return Err("not a leaf");
                    }
                    if freeing {
                        let pa = pte.as_phys_addr();
                        unsafe { SinglePage::free_from_raw(pa as *mut SinglePage) };
                    }
                    pte.data = 0;
                }
                None => {
                    return Err("unmap_pages: pte not found");
                }
            }
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

    fn walk_mut(&mut self, va: usize) -> Option<&mut PageTableEntry> {
        let mut page_table = self as *mut PageTable;

        for level in (1..=2).rev() {
            let pte = unsafe { &page_table.as_ref().unwrap()[get_index(va, level)] };

            if !pte.is_valid() {
                return None;
            }

            page_table = pte.as_page_table();
        }

        unsafe { Some(&mut page_table.as_mut().unwrap()[get_index(va, 0)]) }
    }

    fn walk_alloc(&mut self, va: usize) -> Option<&mut PageTableEntry> {
        let mut page_table = self as *mut PageTable;

        for level in (1..=2).rev() {
            let pte = unsafe { &mut page_table.as_mut().unwrap()[get_index(va, level)] };

            if !pte.is_valid() {
                // The raw page_table pointer is leaked but kept in the page table entry that can calculate later.
                let page_table_ptr = unsafe { PageTable::alloc_into_raw().ok()? };

                pte.set_addr(as_pte_addr(page_table_ptr as usize), PteFlag::VALID);
            }

            page_table = pte.as_page_table();
        }

        unsafe { Some(&mut page_table.as_mut().unwrap()[get_index(va, 0)]) }
    }

    pub fn copy_out(
        &self,
        mut dstva: usize,
        mut src: *const u8,
        mut count: usize,
    ) -> Result<(), &'static str> {
        while count > 0 {
            let va_base = align_down(dstva, PAGESIZE);
            let distance = dstva as usize - va_base;
            let dstpa = unsafe { (self.walk_addr(va_base)? as *mut u8).offset(distance as isize) };

            let n = min(PAGESIZE - distance, count);
            unsafe {
                ptr::copy_nonoverlapping(src, dstpa, n);
            }
            count -= n;
            src = unsafe { src.offset(n as isize) };
            dstva = va_base + PAGESIZE;
        }
        Ok(())
    }

    pub fn copy_in(
        &self,
        mut dst: *mut u8,
        mut srcva: usize,
        mut count: usize,
    ) -> Result<(), &'static str> {
        while count > 0 {
            let va_base = align_down(srcva, PAGESIZE);
            let distance = srcva - va_base;
            let srcpa =
                unsafe { (self.walk_addr(va_base)? as *const u8).offset(distance as isize) };

            let n = min(PAGESIZE - distance, count);
            unsafe {
                ptr::copy_nonoverlapping(srcpa, dst, n);
            }
            count -= n;
            dst = unsafe { dst.offset(n as isize) };
            srcva = va_base + PAGESIZE;
        }
        Ok(())
    }

    /// Copy a null-terminated string from user to kernel.
    /// Copy bytes to dst from virtual address srcva in a given page table,
    /// until a '\0'.
    pub fn copy_in_str(&self, dst: &mut [u8], mut srcva: usize) -> Result<usize, &'static str> {
        let mut i = 0;

        while i < dst.len() {
            let va_base = align_down(srcva, PAGESIZE);
            let distance = srcva - va_base;
            let mut srcpa =
                unsafe { (self.walk_addr(va_base)? as *const u8).offset(distance as isize) };

            let mut count = min(PAGESIZE - distance, dst.len() - 1);
            while count > 0 {
                unsafe {
                    dst[i] = ptr::read(srcpa);
                    if dst[i] == 0 {
                        return Ok(i);
                    }
                    srcpa = srcpa.add(1);
                    i += 1;
                    count -= 1;
                }
            }

            srcva = va_base + PAGESIZE;
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

    #[inline]
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
        self.data >> 10 << 12
    }

    fn free(&mut self) {
        if self.is_valid() {
            if self.is_leaf() {
                // phys memory should already be freed.
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
