use std::{mem};

const SEG_SIZE:usize = 0x10000;
//stupid workaround
const INIT: Option<Box<Page>> = None;

pub struct Memory{
    page_table: Box<[Option<Box<Page>>; SEG_SIZE]>,
}

impl Default for Memory{
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! get_mem {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32) -> $fn_type{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                    (&mut self.get_or_make_page(address).page)[tmp]
            }
        }
    };
}

macro_rules! set_mem {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32, data: $fn_type){
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                    (&mut self.get_or_make_page(address).page)[tmp] = data;
            }
        }
    };
}

#[allow(dead_code)]
impl Memory{

    pub fn new() -> Self{
        Memory{
            page_table: Box::new([INIT; SEG_SIZE])
        }
    }


    #[inline(always)]
    pub fn get_page(&mut self, address: u32) -> Option<&mut Box<Page>> {
        let addr = (address >> 16) as usize;
        self.page_table[addr].as_mut()
    }

    #[inline(always)]
    pub  fn get_or_make_page(&mut self, address: u32) -> &mut Page {
        let addr = (address >> 16) as usize;

        if let Option::None = &mut self.page_table[addr]{
            let page = Box::new(Page::new());
            self.page_table[addr] = Option::Some(page);
        }

        if let Option::Some(page) = & mut self.page_table[addr]{
            return page
        }
        panic!();
    }

    pub fn copy_into_raw<T>(&mut self, address: u32, data: &[T]){
        let size: usize = data.len() * mem::size_of::<T>();
        unsafe { self.copy_into_unsafe(address, mem::transmute(data), 0, size); }
    }

    pub unsafe fn copy_into_unsafe(&mut self, address: u32, data: &[u8], start: usize, end: usize){
        let mut id = start;
        let mut page = self.get_or_make_page(address);
        for im in address..address + (end - start) as u32{
            if im & 0xFFFF == 0 {
                page = self.get_or_make_page(im);
            }
            page.page[(im & 0xFFFF) as usize] = *data.get_unchecked(id);
            id += 1;
        }
    }

    pub fn unload_page_at_address(&mut self, address: u32){
        self.page_table[(address >> 16) as usize] = Option::None;
    }
    pub fn unload_all_pages(&mut self) {
        for i in 0..0xFFFF{
            self.unload_page_at_address(i << 16);
        }
    }

    get_mem!(get_i64_alligned, i64);
    set_mem!(set_i64_alligned, i64);
    get_mem!(get_u64_alligned, u64);
    set_mem!(set_u64_alligned, u64);

    get_mem!(get_i32_alligned, i32);
    set_mem!(set_i32_alligned, i32);
    get_mem!(get_u32_alligned, u32);
    set_mem!(set_u32_alligned, u32);

    get_mem!(get_i16_alligned, i16);
    set_mem!(set_i16_alligned, i16);
    get_mem!(get_u16_alligned, u16);
    set_mem!(set_u16_alligned, u16);

    get_mem!(get_i8, i8);
    set_mem!(set_i8, i8);
    get_mem!(get_u8, u8);
    set_mem!(set_u8, u8);
}


pub struct Page{
    page: [u8; SEG_SIZE],
}

impl Page{
    fn new() -> Self{
        Page{
            page: [0xdf; SEG_SIZE]
        }
    }
}