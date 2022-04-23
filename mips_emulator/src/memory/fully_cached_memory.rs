use std::{mem, ops::{DerefMut}, error::Error};

use crate::{set_mem_alligned, get_mem_alligned, set_mem_alligned_o, get_mem_alligned_o};

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
                self.page_table[(*addresses.next().unwrap()) as usize] = Option::Some(mem::transmute(page));
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
}

// pub struct MemoryGuard<'a>{
//     mem: &'a mut Memory,
//     lock_id: usize,
//     ppref: &'a mut PagePoolRef<Memory>,
// }

// impl<'a> Deref for MemoryGuard<'a>{
//     type Target = Memory;

//     fn deref(&self) -> &Self::Target {
//         self.mem
//     }
// }

// impl<'a> DerefMut for MemoryGuard<'a>{
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.mem
//     }
// }

// impl<'a> Drop for MemoryGuard<'a>{
//     fn drop(&mut self) {
//         self.ppref.id = self.lock_id;
//     }
// }

// impl PagePoolRef<Memory>{
//     pub fn create_guard<'a>(&'a mut self) -> MemoryGuard<'a>{
//         unsafe{

//             let mut lock;
//             match &mut self.inner.as_mut().page_pool{
//                 Some(val) => {
//                     lock = val.get_page_pool();
//                 },
//                 None => {
//                     panic!()
//                 },
//             }
//             let lock_id = self.id;
//             self.id = usize::MAX;
//             lock.last_lock_id = self.id;

//             return MemoryGuard{
//                 mem: self.inner.as_mut(),
//                 lock_id,
//                 ppref: self,
//             }
//         }
//     }
// }

impl Default for PagePoolRef<Memory>{
    fn default() -> Self {
        Memory::new()
    }
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
                let mut mem = lock.add_holder(mem);
                let pool = mem.get_page_pool();
                mem.deref_mut().page_pool = Option::Some(pool);
                return mem;
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
    fn get_or_make_page<'a>(&'a mut self, address: u32) -> &'a mut Page {
        let addr = (address >> 16) as usize;
        //let test = self as *mut Self;
        //we dont need to check if the addr is in bounds since it is always below 2^16
        {
            //println!("address: {}", addr);
            let p =unsafe{self.page_table.get_unchecked_mut(addr)};

            //println!("page: {:p}", p);
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
                                    *p = Option::Some(unsafe{mem::transmute(ok)});
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