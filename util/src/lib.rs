pub mod token;
pub mod tokenizer;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {

        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}


pub fn black_box_1<T>(dummy: T) -> T{
    unsafe {
        let ret = core::ptr::read_volatile(&dummy);
        core::mem::forget(dummy);
        ret
    }
}