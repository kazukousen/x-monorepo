use x86_64::structures::paging::PageTable;
use x86_64::{PhysAddr, VirtAddr};


/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that
/// the complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub unsafe fn translate_addr(addr: VirtAddr, physical_mem_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr(addr, physical_mem_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_mem_offset: VirtAddr) -> Option<PhysAddr> {

    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::page_table::FrameError;

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index(),
    ];

    let (l4_table_frame, _) = Cr3::read();

    let mut frame = l4_table_frame;

    for &index in &table_indexes {
        let virt = physical_mem_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &* table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        }
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
