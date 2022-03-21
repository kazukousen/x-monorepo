use crate::param::{PAGESIZE, PHYSTOP};
use crate::{print, println};
use alloc::alloc::alloc;
use alloc::alloc::Layout;
use core::ptr::write_bytes;
use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub fn heap_init() {
    extern "C" {
        fn end();
    }
    let heap_start: usize = end as usize;
    print!(
        "kalloc: available phys memory[{:#x}, {:#x}]\n",
        heap_start, PHYSTOP
    );
    unsafe {
        ALLOCATOR.lock().init(heap_start, PHYSTOP - heap_start);
    }
    println!("kalloc: init memory done");
}

/// Allocate one 4096-byte page of physical memory.
/// Returns a pointer that the kernel can use.
/// Returns 0 if the memory cannot be allocated.
pub fn kalloc() -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align(PAGESIZE, 4096).unwrap();
        let ptr = alloc(layout);
        write_bytes(ptr, 0x0, PAGESIZE);
        return ptr;
    }
}
