use std::{error::Error, ptr::NonNull, sync::{Arc, Mutex}};

//use crate::{set_mem_alligned, get_mem_alligned, set_mem_alligned_o, get_mem_alligned_o};

use crate::cpu::EmulatorPause;

use super::page_pool::{
    Page, PageImpl, PagePool, PagePoolController, PagedMemoryImpl, PagedMemoryInterface,
    SharedPagePoolMemory, TryLockError, SEG_SIZE,
};

//stupid workaround
const INIT: Option<NonNull<Page>> = None;
pub struct Memory {
    //pub(crate) listener: Option<&'static mut (dyn PagePoolListener + Send + Sync + 'static)>,
    pub(crate) page_pool: Option<Arc<Mutex<PagePoolController>>>,
    //pub(crate) going_to_lock: Option<&'static mut bool>,
    pub(crate) page_table: [Option<NonNull<Page>>; SEG_SIZE],
    emulator: Option<Box<dyn EmulatorPause>>,
}

unsafe impl Sync for Memory {}
unsafe impl Send for Memory {}

impl PagedMemoryImpl for Memory {
    fn init_page_pool_memory(&mut self, notifier: Arc<Mutex<PagePoolController>>) {
        self.page_pool = Option::Some(notifier);
    }

    fn lock(&mut self, initiator: bool, _page_pool: &PagePool) -> Result<(), Box<dyn Error>> {
        if !initiator {
            unsafe {
                self.emulator.as_mut().unwrap().pause();
            }
        }
        Result::Ok(())
    }

    fn unlock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>> {
        let mut iter = page_pool.pool.iter().zip(page_pool.address_mapping.iter());
        let mut next = iter.next();
        for (current_address, current_page) in self.page_table.iter_mut().enumerate() {
            if let Option::Some((page, address)) = next {
                if current_address as u16 == *address {
                    *current_page = Some(page.into());
                    next = iter.next();
                    continue;
                }
            }
            *current_page = Option::None;
        }

        // let pages = page_pool.pool.iter();
        // let addresses = page_pool.address_mapping.iter();
        // for (page, address) in pages.zip(addresses) {
        //     self.page_table[*address as usize] = Option::Some(page.into());
        // }

        if !initiator {
            unsafe {
                self.emulator.as_mut().unwrap().resume();
            }
        }
        Result::Ok(())
    }

    fn get_notifier(&mut self) -> Option<&mut Arc<Mutex<PagePoolController>>> {
        self.page_pool.as_mut()
    }

    fn try_lock(
        &mut self,
        initiator: bool,
        _page_pool: &PagePool,
    ) -> Result<(), TryLockError<Box<dyn Error>>> {
        if !initiator {
            match unsafe { self.emulator.as_mut().unwrap().try_pause(10000) } {
                Ok(_) => Result::Ok(()),
                Err(_) => Result::Err(TryLockError::WouldBlock),
            }
        } else {
            Result::Ok(())
        }
    }

    fn try_unlock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>> {
        self.unlock(initiator, page_pool)
    }
}

impl Default for SharedPagePoolMemory<Memory> {
    fn default() -> Self {
        Memory::new()
    }
}

pub enum TernaryOption<F, S> {
    Option1(F),
    Option2(S),
    None,
}

impl Memory {
    #[inline(never)]
    #[cold]
    unsafe fn create_page(&mut self, addr: u32) -> NonNull<Page> {
        //set_thing(&mut self.going_to_lock);
        match &self.page_pool.clone() {
            Some(val) => {
                let mut val = val.lock().unwrap();
                let val = val.create_page(self, (addr >> 16) as u16);

                let p = self.page_table.get_unchecked_mut(addr as usize >> 16);
                
                if let Ok(ok) = val {
                    *p = Option::Some(ok);
                }
            }
            None => todo!(),
        }

        let p = self.page_table.get_unchecked_mut(addr as usize >> 16);
        //unset_thing(&mut self.going_to_lock);

        match p {
            Some(val) => *val,
            None => std::hint::unreachable_unchecked(),
        }
    }

    pub fn set_emulator_pause(&mut self, pauser: impl EmulatorPause) {
        self.emulator = Option::Some(Box::new(pauser));
    }

    /// # Safety
    ///
    /// ptr must not outlive self
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls the lock method on this trait objects `PagedMemoryImpl`
    pub unsafe fn get_or_make_mut_ptr_to_address(&mut self, address: u32) -> *mut u8 {
        &mut (*self.get_or_make_page(address).page_raw())[(address & 0xFFFF) as usize]
    }

    /// Get a read only slice of contiguous memory withing `SharedPagePool`.
    ///
    /// `start` must be less than `end`
    ///
    /// If the physical addresses between virtual address `start` and `end` are contiguous then a slice is returned else a vec of size `end - start` is allocated and the non contigous memory is copied to the vec
    ///
    /// If any page doesnt exist between virtual addresses `start` and `end` then none is returned
    ///
    /// # Safety
    ///
    /// `Option2` is safe to use as the data is copyed to a seperate allocation
    ///
    /// `Option1` is a slice of contiguous memory within `SharedPagePool` starting at virtual address `start` ending at address `end` (has length `end - start`)
    ///
    /// `Option1`'s `*const [u8]` must not outlive `self` and must be destroyed if `self.lock` is called
    pub unsafe fn slice_vec_or_none(
        &mut self,
        start: u32,
        end: u32,
    ) -> TernaryOption<*const [u8], Vec<u8>> {
        assert!(start < end);
        let len = (end - start) as usize;

        if start & !0xFFFF == end & !0xFFFF {
            if let Some(mut ptr) = self.get_page(start) {
                TernaryOption::Option1(std::slice::from_raw_parts(
                    &(*ptr.page_raw())[start as usize & 0xFFFF],
                    len,
                ))
            } else {
                TernaryOption::None
            }
        } else {
            let mut last = start & !0xFFFF;
            let mut curr = (start & !0xFFFF) + 0x10000;
            let slice = loop {
                {
                    if let (Some(last), Some(curr)) = (self.get_page(last), self.get_page(curr)) {
                        let last = last.as_ptr().cast::<u8>();
                        let curr = curr.as_ptr().cast::<u8>();
                        if last.add(1 << 16) as u64 == curr as u64 {
                            //these pages are consecutive and will be valid if turned into a slice
                        } else {
                            break false;
                        }
                    } else {
                        return TernaryOption::None;
                    }
                }

                last = curr;
                curr += 0x10000;
                if curr == end & !0xFFFF {
                    break true;
                }
            };
            if slice {
                if let Some(mut ptr) = self.get_page(start) {
                    TernaryOption::Option1(std::slice::from_raw_parts(
                        &(*ptr.page_raw())[start as usize & 0xFFFF],
                        len,
                    ))
                } else {
                    TernaryOption::None
                }
            } else {
                let mut vec = Vec::with_capacity(len);

                let mut raw_vec = vec.as_mut_ptr();
                let mut page = self.get_page(start).unwrap();
                let ptr = (*page.page_raw()).as_mut_ptr().add(start as usize & 0xFFFF);

                let page_end = 0x10000 - start as usize;
                if page_end > end as usize {
                    std::ptr::copy_nonoverlapping(ptr, raw_vec, page_end);
                    let mut t_start = (start & 0xFFFF) + 0x10000;
                    loop {
                        let mut page = self.get_page(t_start).unwrap();
                        let ptr = (*page.page_raw()).as_mut_ptr();
                        if t_start == 0xFFFF0000 || t_start + 0x10000 >= end {
                            std::ptr::copy_nonoverlapping(ptr, raw_vec, (end - t_start) as usize);
                            break;
                        } else {
                            std::ptr::copy_nonoverlapping(ptr, raw_vec, 0xFFFF);
                            t_start += 0x10000;
                            raw_vec = raw_vec.add(0x10000);
                        }
                    }
                } else {
                    panic!(); //we should have returned a slice since this data is on a signle page
                              //std::ptr::copy_nonoverlapping(ptr, raw, len);
                }

                vec.set_len(len);

                TernaryOption::Option2(vec)
            }
        }
    }
}

impl<'a> super::page_pool::PagedMemoryInterface<'a> for Memory {
    type Page = NonNull<Page>;

    #[inline(always)]
    unsafe fn get_page(&mut self, address: u32) -> Option<NonNull<Page>> {
        let addr = (address >> 16) as usize;
        *self.page_table.get_unchecked_mut(addr)
    }

    #[inline(always)]
    unsafe fn get_or_make_page(&mut self, address: u32) -> NonNull<Page> {
        //we don't need to check if the addr is in bounds since it is always below 2^16
        let addr = (address >> 16) as usize;

        match self.page_table.get_unchecked_mut(addr) {
            Some(val) => *val,
            None => self.create_page(address),
        }
    }

    unsafe fn try_get_or_make_page(
        &'a mut self,
        address: u32,
    ) -> Result<Self::Page, TryLockError<Box<dyn Error>>> {
        let addr = (address >> 16) as usize;

        match self.page_table.get_unchecked_mut(addr) {
            Some(val) => Ok(*val),
            None => {
                match &self.page_pool.clone() {
                    Some(val) => {
                        let mut val = val.try_lock()?;
                        let val = val.try_create_page(self, (addr >> 16) as u16)?;

                        let p = self.page_table.get_unchecked_mut(addr >> 16);
                        *p = Option::Some(val);
                    }
                    None => todo!(),
                }

                let p = self.page_table.get_unchecked_mut(addr >> 16);

                match p {
                    Some(val) => Ok(*val),
                    None => std::hint::unreachable_unchecked(),
                }
            }
        }
    }

    unsafe fn try_get_page(
        &'a mut self,
        address: u32,
    ) -> Result<Option<Self::Page>, TryLockError<Box<dyn Error>>> {
        Result::Ok(self.get_page(address))
    }
}

unsafe impl<'a> super::page_pool::MemoryDefaultAccess<'a, NonNull<Page>> for Memory {}

#[allow(dead_code)]
impl Memory {
    pub fn new() -> SharedPagePoolMemory<Self> {
        let controller = PagePoolController::new();
        let mut lock = controller.lock();
        match lock.as_mut() {
            Ok(lock) => {
                let mem = box Memory {
                    //going_to_lock: Option::None,
                    page_pool: Option::None,
                    page_table: [INIT; SEG_SIZE],
                    emulator: Option::None,
                    //listener: Option::None,
                };
                lock.add_holder(mem)
            }
            Err(err) => panic!("{err}"),
        }
    }

    // pub fn add_thing(&mut self, thing: &'static mut bool) {
    //     self.going_to_lock = Option::Some(thing);
    // }
    // pub fn remove_thing(&mut self) {
    //     self.going_to_lock = Option::None;
    // }

    // pub fn add_listener(&mut self, listener: &'static mut (dyn PagePoolListener + Send + Sync)) {
    //     self.listener = Option::Some(listener);
    // }
    // pub fn remove_listener(&mut self) {
    //     self.listener = Option::None;
    // }

    pub fn unload_page_at_address(&mut self, address: u32) {
        match &self.page_pool.clone() {
            Some(val) => {
                //set_thing(&mut self.going_to_lock);
                let _ = val.lock().unwrap().remove_page(self, (address >> 16) as u16);
                //unset_thing(&mut self.going_to_lock);
                self.page_table[(address >> 16) as usize] = Option::None;
            }
            None => todo!(),
        }
    }
    pub fn unload_all_pages(&mut self) {
        match &self.page_pool.clone() {
            Some(val) => {
                //set_thing(&mut self.going_to_lock);
                let _ = val.lock().unwrap().remove_all_pages(self);
                //unset_thing(&mut self.going_to_lock);
                self.page_table.iter_mut().for_each(|page| {
                    *page = None;
                });
            }
            None => todo!(),
        }
    }
}
