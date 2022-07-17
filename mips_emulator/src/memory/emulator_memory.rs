use std::{error::Error};

//use crate::{set_mem_alligned, get_mem_alligned, set_mem_alligned_o, get_mem_alligned_o};

use super::page_pool::{Page, PagePoolListener, PagePoolNotifier, SEG_SIZE, PagePool, PagePoolHolder, PagePoolRef, PagePoolController};


//stupid workaround
const INIT: Option<&'static mut Page> = None;
pub struct Memory{
    pub(crate) listener: Option<&'static mut (dyn PagePoolListener + Send + Sync + 'static)>,
    pub(crate) page_pool: Option<PagePoolNotifier>,
    pub(crate) going_to_lock: Option<&'static mut bool>,
    pub(crate) page_table: [Option<&'static mut Page>; SEG_SIZE],
}


impl PagePoolHolder for Memory{

    fn init_holder(&mut self, notifier: PagePoolNotifier) {
        self.page_pool = Option::Some(notifier);   
    }
    
    fn lock(&mut self, initiator: bool, _page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        match &mut self.listener{
            Some(val) => {
                val.lock(initiator)
            },
            None => {
                Result::Ok(())
            },
        }
    }

    fn unlock(&mut self, initiator: bool, page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        for page in self.page_table.iter_mut(){
            *page = Option::None;
        }
        
        let pages = page_pool.pool.iter_mut();
        let mut addresses = page_pool.address_mapping.iter();
        for page in pages{
            unsafe{
                self.page_table[(*addresses.next().unwrap()) as usize] = Option::Some(std::mem::transmute(page));
            }
        }

        match &mut self.listener{
            Some(val) => {
                val.unlock(initiator)
            },
            None => {
                Result::Ok(())
            },
        }
    }

    fn get_notifier(&mut self) -> Option<&mut PagePoolNotifier> {
        self.page_pool.as_mut()
    }
}

impl Default for PagePoolRef<Memory>{
    fn default() -> Self {
        Memory::new()
    }
}

impl<'a> super::page_pool::MemoryDefault<'a, &'a mut Page> for Memory{
    #[inline(always)]
    fn get_page(&mut self, address: u32) -> Option<&mut Page> {
        let addr = (address >> 16) as usize;
        let p =unsafe{self.page_table.get_unchecked_mut(addr)};
        match p{
            Some(val) => Option::Some(val),
            None => Option::None,
        }
    }

    #[inline(always)]
    fn get_or_make_page(&mut self, address: u32) -> &mut Page {
        let addr = (address >> 16) as usize;
        //we dont need to check if the addr is in bounds since it is always below 2^16
        {
            let p =unsafe{self.page_table.get_unchecked_mut(addr)};

            match p{
                Some(val) => return val,
                None => {
                    set_thing(&mut self.going_to_lock);
                    match &self.page_pool{
                        Some(val) => {
                            let mut val = val.get_page_pool();
                            let val = val.create_page(addr as u16);
                            match val{
                                Ok(ok) => {
                                    *p = Option::Some(unsafe{std::mem::transmute(ok)});
                                },
                                Err(_) => {},
                            }
                        },
                        None => todo!(),
                    }
                    unset_thing(&mut self.going_to_lock);

                    match p {
                        Some(val) => return val,
                        None => unsafe { std::hint::unreachable_unchecked() },
                    }  
                },
            }
        }
    }
}

impl<'a> super::page_pool::MemoryDefaultAccess<'a, &'a mut Page> for Memory{

}


#[allow(dead_code)]
impl Memory{

    pub fn new() -> PagePoolRef<Self>{
        let controller = PagePoolController::new();
        let mut lock = controller.lock();
        match lock.as_mut(){
            Ok(lock) => {
                let mem = Memory{
                    going_to_lock: Option::None,
                    page_pool: Option::None,
                    page_table: [INIT; SEG_SIZE],
                    listener: Option::None,
                };
                return lock.add_holder(mem)
            }
            Err(_err) => todo!(),
        }

    }

    pub fn add_thing(&mut self, thing: &'static mut bool){
        self.going_to_lock = Option::Some(thing);
    }
    pub fn remove_thing(&mut self){
        self.going_to_lock = Option::None;
    }

    pub fn add_listener(&mut self, listener: &'static mut (dyn PagePoolListener + Send + Sync)) {
        self.listener = Option::Some(listener);
    }
    pub fn remove_listener(&mut self){
        self.listener = Option::None;
    }

    pub fn unload_page_at_address(&mut self, address: u32){
        match &self.page_pool{
            Some(val) => {
                set_thing(&mut self.going_to_lock);
                let _ = val.get_page_pool().remove_page((address >> 16) as u16);
                unset_thing(&mut self.going_to_lock);
                self.page_table[(address >> 16) as usize] = Option::None;
            },
            None => todo!(),
        }
    }
    pub fn unload_all_pages(&mut self) {
        match &self.page_pool{
            Some(val) => {
                set_thing(&mut self.going_to_lock);
                let _ = val.get_page_pool().remove_all_pages();
                unset_thing(&mut self.going_to_lock);
                for i in 0..(1<<16 -1){
                    self.page_table[i] = Option::None;
                }
            },
            None => todo!(),
        }
    }
}

fn set_thing(thing: &mut Option<&'static mut bool>){
    match thing{
        Some(some) => {
            **some = true;
        },
        None => {},
    }
}
fn unset_thing(thing: &mut Option<&'static mut bool>){
    match thing{
        Some(some) => {
            **some = false;
        },
        None => {},
    }
}