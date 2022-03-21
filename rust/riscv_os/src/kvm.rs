use crate::page_table::{PageTable, PteFlag};
use crate::param::KERNBASE;
use crate::println;

static mut KERNEL_PAGE_TABLE: PageTable = PageTable::empty();

pub unsafe fn init() {
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
}

unsafe fn kvm_map(va: usize, pa: usize, size: usize, perm: PteFlag) {
    println!("kvm_map: va={:#x}, pa={:#x}, size={:#x}", va, pa, size);

    if let Err(err) = KERNEL_PAGE_TABLE.map_pages(va, pa, size, perm) {
        panic!("kvm_map: {}", err)
    }
}
