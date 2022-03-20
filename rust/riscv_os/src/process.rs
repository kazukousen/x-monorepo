use crate::register::tp;

pub fn cpu_id() -> usize {
    unsafe { tp::read() }
}
