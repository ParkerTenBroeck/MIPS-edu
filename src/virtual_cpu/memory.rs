
const SEG_SIZE:usize = 0x10000;
//stupid workaround
const INIT: Option<Box<Page>> = None;

pub struct Memory{
    page_table: [Option<Box<Page>>; SEG_SIZE],
}

#[allow(dead_code)]
impl Memory{

    pub fn new() -> Self{
        Memory{
            page_table: [INIT; SEG_SIZE]
        }
    }


    #[inline(always)]
    fn get_page(& mut self, address: u32) -> & mut Page {
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

    pub fn get_u32(&mut self, address: u32) -> u32{
        use std::convert::TryInto;
        let test = &self.get_page(address).page[(address & 0xFFFFFFFF) as usize..4usize];
        u32::from_ne_bytes(test.try_into().unwrap())
    }
    pub fn get_u16(&mut self, address: u32) -> u16{
        use std::convert::TryInto;
        let test = &self.get_page(address).page[(address & 0xFFFFFFFF) as usize..2usize];
        u16::from_ne_bytes(test.try_into().unwrap())
    }
    pub fn get_u8(&mut self, address: u32) -> u8{
        self.get_page(address).page[(address & 0xFFFFFFFF) as usize]
    }
    pub fn get_i32(&mut self, address: u32) -> i32{
        use std::convert::TryInto;
        let test = &self.get_page(address).page[(address & 0xFFFFFFFF) as usize..4usize];
        i32::from_ne_bytes(test.try_into().unwrap())
    }
    pub fn get_i16(&mut self, address: u32) -> i16{
        use std::convert::TryInto;
        let test = &self.get_page(address).page[(address & 0xFFFFFFFF) as usize..2usize];
        i16::from_ne_bytes(test.try_into().unwrap())
    }
    pub fn get_i8(&mut self, address: u32) -> i8{
        self.get_page(address).page[(address & 0xFFFFFFFF) as usize] as i8
    }
}


struct Page{
    page: [u8; SEG_SIZE],
}

impl Page{
    fn new() -> Self{
        Page{
            page: [0xdf; SEG_SIZE]
        }
    }
}