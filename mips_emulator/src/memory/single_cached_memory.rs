use std::{error::Error, mem};


use crate::{set_mem_alligned, get_mem_alligned, get_mem_alligned_o, set_mem_alligned_o};

use super::page_pool::{PagePoolHolder, PagePool, PagePoolNotifier, Page};


pub struct SingleCachedMemory{
    page_pool: Option<PagePoolNotifier>,
    cache: Option<(u16, &'static mut Page)>,
}

impl SingleCachedMemory{

    fn get_or_make_page(&mut self, page: u32) -> &mut Page{
        let page = (page >> 16) as u16;

        let tmp: &'static mut Option<(u16, &'static mut Page)> = unsafe{mem::transmute(&mut self.cache)};

        if let Option::Some((page, add)) = &mut self.cache{
            if page == page{
                return add;
            }
        }

        match &mut self.page_pool{
            Some(val) => {
                let page_ref = unsafe{mem::transmute(val.get_page_pool().create_page(page).unwrap())};
                *tmp = Option::Some((page, page_ref));
                match tmp{
                    Some(val) => val.1,
                    None => unsafe{std::hint::unreachable_unchecked()},
                }
            },
            None => panic!(),
        }
    }

    fn get_page(&mut self, page: u32) -> Option<&mut Page>{
        let page = (page >> 16) as u16;
        let tmp: &'static mut Option<(u16, &'static mut Page)> = unsafe{mem::transmute(&mut self.cache)};
        if let Option::Some((page, add)) = &mut self.cache{
            if page == page{
                return Option::Some(add);
            }
        }
        match &mut self.page_pool{
            Some(val) => {
                let page_ref: Option<&'static mut Page> = unsafe{mem::transmute(val.get_page_pool().get_page(page))};

                if let Option::Some(page_ref) = page_ref{
                    *tmp = Option::Some((page, page_ref));
                    match tmp{
                        Some(val) => Option::Some(val.1),
                        None => unsafe{std::hint::unreachable_unchecked()},
                    }
                }else{
                    Option::None
                }
            },
            None => panic!(),
        }
    }

    pub fn new() -> Self{
        SingleCachedMemory { page_pool: Option::None, cache: Option::None }
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


impl PagePoolHolder for SingleCachedMemory{
    fn lock(&mut self, initiator: bool, _page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        if !initiator{
            self.cache = Option::None;
        }
        Result::Ok(())
    }

    fn unlock(&mut self, _initiator: bool, _page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        Result::Ok(())
    }

    fn init_holder(&mut self, notifier: PagePoolNotifier) {
        self.page_pool = Option::Some(notifier);
    }
}
