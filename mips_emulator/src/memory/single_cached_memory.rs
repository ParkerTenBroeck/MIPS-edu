use std::{error::Error, ops::DerefMut, ptr::NonNull, sync::Mutex};

//use crate::{set_mem_alligned, get_mem_alligned, get_mem_alligned_o, set_mem_alligned_o};

use super::page_pool::{
    PagedMemoryInterface, MemoryDefaultAccess, Page, PageGuard, PagePool, PagedMemoryImpl, PagePoolNotifier,
};

#[derive(Default)]
pub struct SingleCachedMemory {
    page_pool: Option<PagePoolNotifier>,
    cache: Mutex<Option<(u16, NonNull<Page>)>>,
}

unsafe impl Sync for SingleCachedMemory {}
unsafe impl Send for SingleCachedMemory {}

//pub type Ret<'a> = PageGuard<'a, Option<(u16, &'static mut Page)>>;

macro_rules! page_pool {
    // `()` indicates that the macro takes no argument.
    ($func_name:ident) => {
        // The macro will expand into the contents of this block.
        match &mut $func_name.page_pool {
            Option::Some(val) => val,
            Option::None => panic!(),
        }
    };
}

impl<'a> PagedMemoryInterface<'a> for SingleCachedMemory {

    type Page = PageGuard<'a>;

    unsafe fn get_or_make_page(&'a mut self, page_id: u32) -> PageGuard<'a> {
        let page_id = (page_id >> 16) as u16;

        //let tmp: &'static mut Option<(u16, &'static mut Page)> = &mut self.cache;

        //let mut guard = self.cache.lock().unwrap();
        //let unsafe_guard = (&mut guard) as *mut MutexGuard<'_, Option<(u16, &'static mut Page)>>;
        if let Option::Some((page_id_cache, page)) = *self.cache.lock().unwrap() {
            if page_id == page_id_cache {
                let tmp = page_pool!(self);
                let tmp = tmp.create_controller_guard(page);
                return tmp;
            }
        }

        match &mut self.page_pool {
            Some(val) => {
                let page_ref = val.get_page_pool().create_page(page_id).unwrap();

                let mut lock = self.cache.lock().unwrap();
                let tmp = lock.deref_mut();

                // let mut t1 = val.get_page_pool();
                // let t2 = t1.create_page(page_id);
                // let tmp_pg = t2.unwrap();

                *tmp = Option::Some((page_id, page_ref));
                match *tmp {
                    Some((_page_id, page)) => {
                        page_pool!(self).create_controller_guard(page)
                    },
                    None => std::hint::unreachable_unchecked(),
                }
            }
            None => panic!(),
        }
    }

    #[inline(always)]
    unsafe fn get_page(&'a mut self, page_id: u32) -> Option<PageGuard<'a>> {
        let page_id = (page_id >> 16) as u16;

        let mut page_pool = page_pool!(self).get_page_pool();
        if let Option::Some((page_id_cache, page)) = *self.cache.lock().unwrap() {
            if page_id == page_id_cache {
                return Option::Some(PagePoolNotifier::new_controller_guard(page_pool, page));
            }
        }

        let page_ref: Option<NonNull<Page>> = page_pool.get_page(page_id);
        let mut lock = self.cache.lock().unwrap();
        let tmp = lock.deref_mut();

        if let Option::Some(page_ref) = page_ref {
            *tmp = Option::Some((page_id, page_ref));
            match *tmp {
                Some((_page_id, page)) => {
                    Option::Some(PagePoolNotifier::new_controller_guard(page_pool, page))
                }
                None => std::hint::unreachable_unchecked(),
            }
        } else {
            Option::None
        }
    }
}

unsafe impl<'a> MemoryDefaultAccess<'a, PageGuard<'a>> for SingleCachedMemory {}

impl SingleCachedMemory {
    pub fn get_page_pool(&mut self) -> &mut PagePoolNotifier {
        match &mut self.page_pool {
            Some(val) => val,
            None => panic!(),
        }
    }

    pub fn new() -> Box<Self> {
        box SingleCachedMemory {
            page_pool: Option::None,
            cache: Mutex::new(Option::None),
        }
    }
}

impl PagedMemoryImpl for SingleCachedMemory {
    fn get_notifier(&mut self) -> Option<&mut PagePoolNotifier> {
        self.page_pool.as_mut()
    }

    fn lock(&mut self, initiator: bool, _page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        if !initiator {
            *self.cache.lock().unwrap().deref_mut() = Option::None;
        }
        Result::Ok(())
    }

    fn unlock(
        &mut self,
        _initiator: bool,
        _page_pool: &mut PagePool,
    ) -> Result<(), Box<dyn Error>> {
        Result::Ok(())
    }

    fn init_notifier(&mut self, notifier: PagePoolNotifier) {
        self.page_pool = Option::Some(notifier);
    }
}

mod tests {
    #[test]
    fn interlock_test() {
        use std::sync::{Arc, Mutex};

        use crate::memory::page_pool::PagePoolController;

        impl Drop for SingleCachedMemory {
            fn drop(&mut self) {
                println!("DROPPPING MEMORY");
            }
        }

        use super::*;

        let pool = PagePoolController::new();
        let mem1 = SingleCachedMemory::new();
        let mem2 = SingleCachedMemory::new();

        let mut mem1 = pool.lock().as_mut().unwrap().add_holder(mem1);
        let mut mem2 = pool.lock().as_mut().unwrap().add_holder(mem2);

        let step = Arc::new(Mutex::from(0));
        let step1 = step.clone();

        let thread1 = std::thread::spawn(move || {
            unsafe {
                let _page = mem1.get_or_make_page(0);
                println!("Thread has page from mem1");
                *step.lock().unwrap() = 1;
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            println!("has dropped page from mem1");
        });

        let step = step1;
        let thread2 = std::thread::spawn(move || {
            while *step.lock().unwrap() != 1 {}
            unsafe {
                println!("trying to get page from mem2");
                let _page = mem2.get_or_make_page(1);
                println!("has gotten page from mem2");
            }
            println!("has dropped page from mem2");
        });

        let _ = thread1.join();
        let _ = thread2.join();
    }
}
