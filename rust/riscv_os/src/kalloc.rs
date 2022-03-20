use crate::param::PHYSTOP;
use crate::{print, println};
use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn kinit() {
    extern "C" {
        fn end();
    }
    let heap_start: usize = unsafe { end as usize };
    print!(
        "kalloc: available phys memory[{:#x}, {:#x}]\n",
        heap_start, PHYSTOP
    );
    unsafe {
        ALLOCATOR.lock().init(heap_start, PHYSTOP - heap_start);
    }
    println!("kalloc: init memory done");
}
