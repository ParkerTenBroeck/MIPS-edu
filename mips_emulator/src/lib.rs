#![feature(box_syntax)]

pub mod memory;
pub mod cpu;

pub fn black_box<T>(dummy: T) -> T{
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}