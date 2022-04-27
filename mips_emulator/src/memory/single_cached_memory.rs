use std::{error::Error, mem};


//use crate::{set_mem_alligned, get_mem_alligned, get_mem_alligned_o, set_mem_alligned_o};

use super::{page_pool::{PagePoolHolder, PagePool, PagePoolNotifier, Page, MemoryDefault}};


pub struct SingleCachedMemory{
    page_pool: Option<PagePoolNotifier>,
    cache: Option<(u16, &'static mut Page)>,
}

impl MemoryDefault for SingleCachedMemory{
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
    #[inline(always)]
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
}

impl SingleCachedMemory{

    pub fn new() -> Self{
        SingleCachedMemory { page_pool: Option::None, cache: Option::None }
    }
}


impl PagePoolHolder for SingleCachedMemory{

    fn get_notifier(&mut self) -> Option<&mut PagePoolNotifier> {
        self.page_pool.as_mut()
    }

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
