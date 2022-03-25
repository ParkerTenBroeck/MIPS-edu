use std::mem;

const SEG_SIZE:usize = 0x10000;
//stupid workaround
const INIT: Option<Box<Page>> = None;

//maybe use a pool?
pub struct Memory{
    page_table: Box<[Option<Box<Page>>; SEG_SIZE]>,
}

impl Default for Memory{
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! get_mem_alligned {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32) -> $fn_type{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                *mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                    (&mut self.get_or_make_page(address).page).get_unchecked(tmp)
            }
        }
    };
}

macro_rules! set_mem_alligned {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32, data: $fn_type){
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                *mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                    (&mut self.get_or_make_page(address).page).get_unchecked_mut(tmp) = data;
            }
        }
    };
}

macro_rules! get_mem_alligned_o {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32) -> Option<$fn_type>{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                match &mut self.get_page(address){
                    Option::Some(val) => {
                        return Option::Some(
                            mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                            (&mut val.page)[tmp]);
                    }
                    Option::None => {
                        return Option::None;
                    }
                } 
                
            }
        }
    };
}

macro_rules! set_mem_alligned_o {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32, data: $fn_type) -> Result<(), ()>{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            match self.get_page(address){
                Option::Some(val) => {
                    unsafe{
                        mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                            (&mut val.page)[tmp] = data;
                    }
                    return Result::Ok(());
                }
                Option::None => {
                    return Result::Err(());
                }
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
    pub fn get_or_make_page<'a>(&'a mut self, address: u32) -> &'a mut Page {
        let addr = (address >> 16) as usize;
        //we dont need to check if the addr is in bounds since it is always below 2^16
        let p =unsafe{self.page_table.get_unchecked_mut(addr)};

        match p{
            Some(val) => return val,
            None => {
                let page = Box::new(Page::new());
                *p = Option::Some(page);  
                match p {
                    Some(val) => return val,
                    None => unsafe { std::hint::unreachable_unchecked() },
                }
            },
        }
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

    get_mem_alligned!(get_i64_alligned, i64);
    set_mem_alligned!(set_i64_alligned, i64);
    get_mem_alligned!(get_u64_alligned, u64);
    set_mem_alligned!(set_u64_alligned, u64);

    get_mem_alligned!(get_i32_alligned, i32);
    set_mem_alligned!(set_i32_alligned, i32);
    get_mem_alligned!(get_u32_alligned, u32);
    set_mem_alligned!(set_u32_alligned, u32);

    get_mem_alligned!(get_i16_alligned, i16);
    set_mem_alligned!(set_i16_alligned, i16);
    get_mem_alligned!(get_u16_alligned, u16);
    set_mem_alligned!(set_u16_alligned, u16);

    get_mem_alligned!(get_i8, i8);
    set_mem_alligned!(set_i8, i8);
    get_mem_alligned!(get_u8, u8);
    set_mem_alligned!(set_u8, u8);

    get_mem_alligned_o!(get_i64_alligned_o, i64);
    set_mem_alligned_o!(set_i64_alligned_o, i64);
    get_mem_alligned_o!(get_u64_alligned_o, u64);
    set_mem_alligned_o!(set_u64_alligned_o, u64);

    get_mem_alligned_o!(get_i32_alligned_o, i32);
    set_mem_alligned_o!(set_i32_alligned_o, i32);
    get_mem_alligned_o!(get_u32_alligned_o, u32);
    set_mem_alligned_o!(set_u32_alligned_o, u32);

    get_mem_alligned_o!(get_i16_alligned_o, i16);
    set_mem_alligned_o!(set_i16_alligned_o, i16);
    get_mem_alligned_o!(get_u16_alligned_o, u16);
    set_mem_alligned_o!(set_u16_alligned_o, u16);

    get_mem_alligned_o!(get_i8_o, i8);
    set_mem_alligned_o!(set_i8_o, i8);
    get_mem_alligned_o!(get_u8_o, u8);
    set_mem_alligned_o!(set_u8_o, u8);
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