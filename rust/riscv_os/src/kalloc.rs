use crate::param::PHYSTOP;
use crate::println;
use alloc::alloc::Layout;
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
    println!(
        "kalloc: available phys memory[{:#x}, {:#x}]",
        heap_start, PHYSTOP
    );
    unsafe {
        ALLOCATOR.lock().init(heap_start, PHYSTOP - heap_start);
    }
    println!("kalloc: init memory done");
}
