use std::{error::Error, ops::DerefMut, ptr::NonNull, sync::Mutex};

use mips_emulator::memory::page_pool::{
    MemoryDefault, MemoryDefaultAccess, NotifierGuard, Page, PageGuard, PagePool, PagePoolHolder,
    PagePoolNotifier,
};

use crate::platform::sync::PlatSpecificLocking;

//use crate::{set_mem_alligned, get_mem_alligned, get_mem_alligned_o, set_mem_alligned_o};

#[derive(Default)]
pub struct SingleCachedPlatSpinMemory {
    page_pool: Option<PagePoolNotifier>,
    cache: Mutex<Option<(u16, NonNull<Page>)>>,
}

unsafe impl Sync for SingleCachedPlatSpinMemory {}
unsafe impl Send for SingleCachedPlatSpinMemory {}

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

impl<'a> MemoryDefault<'a, PageGuard<'a>> for SingleCachedPlatSpinMemory {
    unsafe fn get_or_make_page(&'a mut self, page_id: u32) -> PageGuard<'a> {
        let page_id = (page_id >> 16) as u16;

        let notifire = self.page_pool.as_mut().unwrap();
        let raw = notifire.get_raw_pool();
        let raw_lock = raw.plat_lock().unwrap();
        let mut guard = NotifierGuard::from_raw(raw_lock, notifire);

        if let Option::Some((page_id_cache, page)) = *self.cache.plat_lock().unwrap() {
            if page_id == page_id_cache {
                return PagePoolNotifier::new_controller_guard(guard, page);
            }
        }

        let page_ref = guard.create_page(page_id).unwrap();

        let mut lock = self.cache.plat_lock().unwrap();
        let tmp = lock.deref_mut();

        *tmp = Option::Some((page_id, page_ref));
        match *tmp {
            Some((_page_id, page)) => PagePoolNotifier::new_controller_guard(guard, page),
            None => std::hint::unreachable_unchecked(),
        }
    }

    #[inline(always)]
    unsafe fn get_page(&'a mut self, page_id: u32) -> Option<PageGuard<'a>> {
        let page_id = (page_id >> 16) as u16;

        let mut page_pool = page_pool!(self).get_page_pool();
        if let Option::Some((page_id_cache, page)) = *self.cache.plat_lock().unwrap() {
            if page_id == page_id_cache {
                return Option::Some(PagePoolNotifier::new_controller_guard(page_pool, page));
            }
        }

        let page_ref: Option<NonNull<Page>> = page_pool.get_page(page_id);
        let mut lock = self.cache.plat_lock().unwrap();
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

unsafe impl<'a> MemoryDefaultAccess<'a, PageGuard<'a>> for SingleCachedPlatSpinMemory {}

impl SingleCachedPlatSpinMemory {
    pub fn get_page_pool(&mut self) -> &mut PagePoolNotifier {
        match &mut self.page_pool {
            Some(val) => val,
            None => panic!(),
        }
    }
}

impl PagePoolHolder for SingleCachedPlatSpinMemory {
    fn get_notifier(&mut self) -> Option<&mut PagePoolNotifier> {
        self.page_pool.as_mut()
    }

    fn lock(&mut self, initiator: bool, _page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        if !initiator {
            *self.cache.plat_lock().unwrap().deref_mut() = Option::None;
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

    fn init_holder(&mut self, notifier: PagePoolNotifier) {
        self.page_pool = Option::Some(notifier);
    }
}
