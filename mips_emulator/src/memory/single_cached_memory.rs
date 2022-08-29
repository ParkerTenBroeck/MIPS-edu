use std::{error::Error, ops::DerefMut, ptr::NonNull, sync::Mutex};

//use crate::{set_mem_alligned, get_mem_alligned, get_mem_alligned_o, set_mem_alligned_o};

use super::page_pool::{
    MemoryDefaultAccess, Page, PageGuard, PagePool, PagedMemoryImpl, PagedMemoryInterface,
    SharedPagePool,
};

#[derive(Default)]
pub struct SingleCachedMemory {
    page_pool: Option<SharedPagePool>,
    cache: Mutex<Option<(u16, NonNull<Page>)>>,
}

unsafe impl Sync for SingleCachedMemory {}
unsafe impl Send for SingleCachedMemory {}

impl<'a> PagedMemoryInterface<'a> for SingleCachedMemory {
    type Page = PageGuard<'a>;

    unsafe fn get_or_make_page(&'a mut self, page_id: u32) -> PageGuard<'a> {
        let page_id = (page_id >> 16) as u16;

        let mut pool_guard = self.page_pool.as_mut().unwrap().get_page_pool();

        if let Option::Some((page_id_cache, page)) = *self.cache.lock().unwrap() {
            if page_id == page_id_cache {
                return SharedPagePool::new_controller_guard(pool_guard, page);
            }
        }

        let page_ref = pool_guard.create_page(page_id).unwrap();

        let mut lock = self.cache.lock().unwrap();
        let tmp = lock.deref_mut();

        *tmp = Option::Some((page_id, page_ref));
        match *tmp {
            Some((_page_id, page)) => SharedPagePool::new_controller_guard(pool_guard, page),
            None => std::hint::unreachable_unchecked(),
        }
    }

    #[inline(always)]
    unsafe fn get_page(&'a mut self, page_id: u32) -> Option<PageGuard<'a>> {
        let page_id = (page_id >> 16) as u16;

        let mut pool_guard = self.page_pool.as_mut().unwrap().get_page_pool();

        if let Option::Some((page_id_cache, page)) = *self.cache.lock().unwrap() {
            if page_id == page_id_cache {
                return Option::Some(SharedPagePool::new_controller_guard(pool_guard, page));
            }
        }

        let page = pool_guard.get_page(page_id);

        match page {
            Some(page) => {
                *self.cache.lock().unwrap() = Some((page_id, page));
                Option::Some(SharedPagePool::new_controller_guard(pool_guard, page))
            }
            None => {
                *self.cache.lock().unwrap() = None;
                None
            }
        }
    }
}

unsafe impl<'a> MemoryDefaultAccess<'a, PageGuard<'a>> for SingleCachedMemory {}

impl SingleCachedMemory {
    pub fn get_page_pool(&mut self) -> &mut SharedPagePool {
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
    fn get_notifier(&mut self) -> Option<&mut SharedPagePool> {
        self.page_pool.as_mut()
    }

    fn lock(&mut self, initiator: bool, _page_pool: &PagePool) -> Result<(), Box<dyn Error>> {
        if !initiator {
            *self.cache.lock().unwrap().deref_mut() = Option::None;
        }
        Result::Ok(())
    }

    fn unlock(&mut self, _initiator: bool, _page_pool: &PagePool) -> Result<(), Box<dyn Error>> {
        Result::Ok(())
    }

    fn init_notifier(&mut self, notifier: SharedPagePool) {
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
