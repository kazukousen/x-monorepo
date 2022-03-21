use crate::page_table::{PageTable, PteFlag};
use crate::param::{
    CLINT, CLINT_MAP_SIZE, KERNBASE, PHYSTOP, PLIC, PLIC_MAP_SIZE, UART0, UART0_MAP_SIZE, VIRTIO0,
    VIRTIO0_MAP_SIZE,
};
use crate::println;
use crate::register::satp;
use core::arch::asm;

static mut KERNEL_PAGE_TABLE: PageTable = PageTable::empty();

pub unsafe fn init_hart() {
    satp::write(KERNEL_PAGE_TABLE.as_satp());
    asm!("sfence.vma zero, zero");
}

pub unsafe fn init() {
    // uart registers
    kvm_map(UART0, UART0, UART0_MAP_SIZE, PteFlag::Read | PteFlag::Write);

    // virtio mmio disk interface
    kvm_map(
        VIRTIO0,
        VIRTIO0,
        VIRTIO0_MAP_SIZE,
        PteFlag::Read | PteFlag::Write,
    );

    // CLINT
    kvm_map(CLINT, CLINT, CLINT_MAP_SIZE, PteFlag::Read | PteFlag::Write);

    // PLIC
    kvm_map(PLIC, PLIC, PLIC_MAP_SIZE, PteFlag::Read | PteFlag::Write);

    extern "C" {
        fn _etext();
    }
    let etext = _etext as usize;

    // map kernel text executable and read-only.
    kvm_map(
        KERNBASE,
        KERNBASE,
        etext - KERNBASE,
        PteFlag::Read | PteFlag::Exec,
    );

    // map kernel data and the physical RAM we'll make use of.
    kvm_map(
        etext,
        etext,
        PHYSTOP - etext,
        PteFlag::Read | PteFlag::Write,
    );

    extern "C" {
        fn _trampoline();
    }
    let trampoline = _trampoline as usize;

    println!(
        "trampoline={:#x}, etext={:#x}, satp={:#x}",
        trampoline,
        etext,
        KERNEL_PAGE_TABLE.as_satp()
    );
}

unsafe fn kvm_map(va: usize, pa: usize, size: usize, perm: PteFlag) {
    println!("kvm_map: va={:#x}, pa={:#x}, size={:#x}", va, pa, size);

    if let Err(err) = KERNEL_PAGE_TABLE.map_pages(va, pa, size, perm) {
        panic!("kvm_map: {}", err)
    }
}
