#![feature(box_syntax)]
#![feature(core_intrinsics)]

pub mod cpu;
pub mod memory;

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}
