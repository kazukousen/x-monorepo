use crate::page_table::{PageTable, PteFlag};
use crate::param::{
    CLINT, CLINT_MAP_SIZE, KERNBASE, PAGESIZE, PHYSTOP, PLIC, PLIC_MAP_SIZE, TRAMPOLINE, UART0,
    UART0_MAP_SIZE, VIRTIO0, VIRTIO0_MAP_SIZE,
};
use crate::register::satp;
use crate::QEMU_TEST0;
use core::arch::asm;

static mut KERNEL_PAGE_TABLE: PageTable = PageTable::empty();

pub unsafe fn init_hart() {
    satp::write(KERNEL_PAGE_TABLE.as_satp());
    asm!("sfence.vma zero, zero");
}

pub unsafe fn init() {
    // uart registers
    kvm_map(UART0, UART0, UART0_MAP_SIZE, PteFlag::READ | PteFlag::WRITE);

    // virtio mmio disk interface
    kvm_map(
        VIRTIO0,
        VIRTIO0,
        VIRTIO0_MAP_SIZE,
        PteFlag::READ | PteFlag::WRITE,
    );

    kvm_map(
        QEMU_TEST0,
        QEMU_TEST0,
        PAGESIZE,
        PteFlag::READ | PteFlag::WRITE,
    );

    // CLINT
    kvm_map(CLINT, CLINT, CLINT_MAP_SIZE, PteFlag::READ | PteFlag::WRITE);

    // PLIC
    kvm_map(PLIC, PLIC, PLIC_MAP_SIZE, PteFlag::READ | PteFlag::WRITE);

    extern "C" {
        fn _etext();
    }
    let etext = _etext as usize;

    // map kernel text executable and read-only.
    kvm_map(
        KERNBASE,
        KERNBASE,
        etext - KERNBASE,
        PteFlag::READ | PteFlag::EXEC,
    );

    // map kernel data and the physical RAM we'll make use of.
    kvm_map(
        etext,
        etext,
        PHYSTOP - etext,
        PteFlag::READ | PteFlag::WRITE,
    );

    extern "C" {
        fn trampoline();
    }

    kvm_map(
        TRAMPOLINE,
        trampoline as usize,
        PAGESIZE,
        PteFlag::READ | PteFlag::EXEC,
    );
}

pub unsafe fn kvm_map(va: usize, pa: usize, size: usize, perm: PteFlag) {
    if let Err(err) = KERNEL_PAGE_TABLE.map_pages(va, pa, size, perm) {
        panic!("kvm_map: {}", err)
    }
}
