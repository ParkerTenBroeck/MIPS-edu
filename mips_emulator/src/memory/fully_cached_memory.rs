#![allow(deprecated)]

use std::{error::Error, ptr::NonNull};

use super::page_pool::{Page, PagePoolListener, PagePoolNotifier, SEG_SIZE, PagePool, PagePoolHolder, PagePoolRef, PagePoolController};


//stupid workaround
const INIT: Option<NonNull<Page>> = None;

#[deprecated]
pub struct FullyCachedMemory{
    pub(crate) listener: Option<&'static mut (dyn PagePoolListener + Send + Sync + 'static)>,
    pub(crate) page_pool: Option<PagePoolNotifier>,
    pub(crate) page_table: [Option<NonNull<Page>>; SEG_SIZE],
}
unsafe impl Send for FullyCachedMemory{

}
unsafe impl Sync for FullyCachedMemory{
    
}

impl PagePoolHolder for FullyCachedMemory{

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
            self.page_table[(*addresses.next().unwrap()) as usize] = Option::Some(page.into());
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

impl Default for PagePoolRef<FullyCachedMemory>{
    fn default() -> Self {
        FullyCachedMemory::new()
    }
}

impl<'a> super::page_pool::MemoryDefault<'a, &'a mut Page> for FullyCachedMemory{
    #[inline(always)]
    unsafe fn get_page(&mut self, address: u32) -> Option<&mut Page> {
        let addr = (address >> 16) as usize;
        match *self.page_table.get_unchecked_mut(addr){
            Some(mut val) => Option::Some(val.as_mut()),
            None => Option::None,
        }
    }

    #[inline(always)]
    unsafe fn get_or_make_page(&mut self, address: u32) -> &mut Page {
        let addr = (address >> 16) as usize;
        //we dont need to check if the addr is in bounds since it is always below 2^16
        {
            let p = self.page_table.get_unchecked_mut(addr);
            match *p{
                Some(mut val) => return val.as_mut(),
                None => {
                    
                    match &self.page_pool{
                        Some(val) => {
                            let mut val = val.get_page_pool();
                            let val = val.create_page(addr as u16);
                            match val{
                                Ok(ok) => {
                                    *p = Option::Some(ok);
                                },
                                Err(_) => {},
                            }
                        },
                        None => todo!(),
                    }
                    

                    match *p {
                        Some(mut val) => return val.as_mut(),
                        None => std::hint::unreachable_unchecked() ,
                    }  
                },
            }
        }
    }
}


#[allow(dead_code)]
impl FullyCachedMemory{

    pub fn new() -> PagePoolRef<Self>{
        let controller = PagePoolController::new();
        let mut lock = controller.lock();
        match lock.as_mut(){
            Ok(lock) => {
                let mem = box FullyCachedMemory{
                    page_pool: Option::None,
                    page_table: [INIT; SEG_SIZE],
                    listener: Option::None,
                };
                return lock.add_holder(mem)
            }
            Err(_err) => todo!(),
        }
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
                
                let _ = val.get_page_pool().remove_page((address >> 16) as u16);
                
                self.page_table[(address >> 16) as usize] = Option::None;
            },
            None => todo!(),
        }
    }
    pub fn unload_all_pages(&mut self) {
        match &self.page_pool{
            Some(val) => {
                
                let _ = val.get_page_pool().remove_all_pages();
                
                for i in 0..(1<<16 -1){
                    self.page_table[i] = Option::None;
                }
            },
            None => todo!(),
        }
    }
}