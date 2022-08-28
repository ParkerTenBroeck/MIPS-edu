use std::{
    error::Error,
    fmt::Debug,
    mem,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

pub const SEG_SIZE: usize = 0x10000;

#[repr(align(0x10000))]
pub struct Page {
    pub page: [u8; SEG_SIZE],
}

impl Page {
    fn new() -> Self {
        Page {
            page: [0x00; SEG_SIZE],
        }
    }
}

pub trait PageImpl {
    unsafe fn page_raw(&mut self) -> *mut [u8; SEG_SIZE];
}

impl PageImpl for *mut Page {
    unsafe fn page_raw(&mut self) -> *mut [u8; SEG_SIZE] {
        &mut (**self).page
    }
}

impl PageImpl for NonNull<Page> {
    unsafe fn page_raw(&mut self) -> *mut [u8; SEG_SIZE] {
        &mut self.as_mut().page
    }
}

impl PageImpl for &mut Page {
    unsafe fn page_raw(&mut self) -> *mut [u8; SEG_SIZE] {
        &mut self.page
    }
}

pub trait PagedMemoryImpl {
    fn init_notifier(&mut self, _notifier: PagePoolNotifier) {}
    fn get_notifier(&mut self) -> Option<&mut PagePoolNotifier>;
    fn lock(&mut self, initiator: bool, page_pool: &mut PagePool) -> Result<(), Box<dyn Error>>;
    fn unlock(&mut self, initiator: bool, page_pool: &mut PagePool) -> Result<(), Box<dyn Error>>;
}

pub trait PagePoolListener {
    fn lock(&mut self, initiator: bool) -> Result<(), Box<dyn Error>>;
    fn unlock(&mut self, initiator: bool) -> Result<(), Box<dyn Error>>;
}

//------------------------------------------------------------------------------------------------------

pub type PageGuard<'a> = ControllerGuard<'a, NonNull<Page>>;

impl<'a> PageImpl for PageGuard<'a> {
    unsafe fn page_raw(&mut self) -> *mut [u8; SEG_SIZE] {
        &mut self.as_mut().page
    }
}

//------------------------------------------------------------------------------------------------------
pub struct ControllerGuard<'a, T> {
    _guard: NotifierGuard<'a>,
    pub data: T,
}
impl<'a, T> std::ops::Deref for ControllerGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<'a, T> std::ops::DerefMut for ControllerGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

//------------------------------------------------------------------------------------------------------

pub struct PagePoolNotifier {
    page_pool: Arc<Mutex<PagePoolController>>,
    id: usize,
}

impl PagePoolNotifier {
    pub unsafe fn get_raw_pool(&self) -> &Arc<Mutex<PagePoolController>> {
        &self.page_pool
    }

    pub fn get_page_pool(&self) -> NotifierGuard {
        let mut controller = self.page_pool.lock().unwrap();
        controller.last_lock_id = self.id;
        NotifierGuard { guard: controller }
    }

    pub fn clone_page_pool_mutex(&self) -> Arc<Mutex<PagePoolController>> {
        self.page_pool.clone()
    }

    pub fn create_controller_guard<T>(&self, data: T) -> ControllerGuard<'_, T> {
        ControllerGuard {
            _guard: self.get_page_pool(),
            data,
        }
    }

    pub fn new_controller_guard<'a, T>(
        guard: NotifierGuard<'a>,
        data: T,
    ) -> ControllerGuard<'a, T> {
        ControllerGuard {
            _guard: guard,
            data,
        }
    }
}

//------------------------------------------------------------------------------------------------------

pub struct NotifierGuard<'a> {
    guard: MutexGuard<'a, PagePoolController>,
}

impl<'a> NotifierGuard<'a> {
    pub unsafe fn from_raw(
        mut guard: MutexGuard<'a, PagePoolController>,
        notifier: &PagePoolNotifier,
    ) -> Self {
        guard.last_lock_id = notifier.id;
        Self { guard }
    }
}

impl<'a> Drop for NotifierGuard<'a> {
    fn drop(&mut self) {
        self.guard.last_lock_id = usize::MAX;
    }
}
impl<'a> Deref for NotifierGuard<'a> {
    type Target = MutexGuard<'a, PagePoolController>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}
impl<'a> DerefMut for NotifierGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

//------------------------------------------------------------------------------------------------------

pub struct PagePoolRef<T: PagedMemoryImpl + Send + Sync> {
    inner: NonNull<T>,
    page_pool: Arc<Mutex<PagePoolController>>,
    id: usize,
}

unsafe impl<T: PagedMemoryImpl + Send + Sync> Send for PagePoolRef<T> {}

unsafe impl<T: PagedMemoryImpl + Send + Sync> Sync for PagePoolRef<T> {}

impl<T: PagedMemoryImpl + Send + Sync> Drop for PagePoolRef<T> {
    fn drop(&mut self) {
        self.page_pool
            .lock()
            .as_mut()
            .unwrap()
            .remove_holder(self.id);
    }
}

impl<T: PagedMemoryImpl + Send + Sync> PagePoolRef<T> {
    fn get_inner_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { self.inner.as_mut() }
    }

    fn get_inner<'a>(&'a self) -> &'a T {
        unsafe { self.inner.as_ref() }
    }

    pub fn get_page_pool(&self) -> PagePoolNotifier {
        PagePoolNotifier {
            page_pool: self.page_pool.to_owned(),
            id: self.id,
        }
    }
}

impl<T: PagedMemoryImpl + Send + Sync> Deref for PagePoolRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_inner()
    }
}

impl<T: PagedMemoryImpl + Send + Sync> DerefMut for PagePoolRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_inner_mut()
    }
}

//------------------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct PagePool {
    pub pool: Vec<Page>,
    pub address_mapping: Vec<u16>,
}

//------------------------------------------------------------------------------------------------------

pub struct PagePoolController {
    page_pool: PagePool,
    holders: Vec<(usize, NonNull<dyn PagedMemoryImpl + Send + Sync>)>,
    myself: Weak<Mutex<PagePoolController>>,
    last_lock_id: usize,
}

impl Debug for PagePoolController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PagePoolController")
            .field("holders", &self.holders)
            .field("myself", &self.myself)
            .finish()
    }
}

unsafe impl Send for PagePoolController {}
unsafe impl Sync for PagePoolController {}

impl PagePoolController {
    pub fn new() -> Arc<Mutex<Self>> {
        let arc;
        unsafe {
            let tmp = mem::MaybeUninit::<PagePoolController>::zeroed();
            arc = Arc::new(Mutex::new(tmp.assume_init()));
            let weak = Arc::downgrade(&arc);

            match arc.lock().as_mut() {
                Ok(val) => {
                    let test = val.deref_mut() as *mut PagePoolController;
                    test.write(PagePoolController {
                        page_pool: PagePool::default(),
                        holders: Vec::new(),
                        myself: weak,
                        last_lock_id: 0,
                    });
                }
                Err(_err) => {
                    panic!();
                }
            }
        }
        arc
    }

    pub fn add_holder<T: PagedMemoryImpl + Send + Sync + 'static>(
        &mut self,
        holder: Box<T>,
    ) -> PagePoolRef<T> {
        let mut id: usize = 0;

        for holder in &self.holders {
            if holder.0 >= id {
                id = holder.0 + 1;
            }
        }

        let mut ptr = NonNull::new(Box::into_raw(holder)).unwrap();

        self.holders.push((id, ptr));

        let ppref = PagePoolRef {
            inner: ptr,
            page_pool: self.myself.upgrade().unwrap(),
            id,
        };

        unsafe { ptr.as_mut() }.init_notifier(ppref.get_page_pool());

        ppref
    }

    pub fn remove_holder(&mut self, id: usize) {
        let index = self.holders.iter().position(|i| id == i.0);
        //self.holders.iter_mut().f
        match index {
            Some(index) => {
                let item = self.holders.remove(index);
                let item = item.1;
                unsafe {
                    item.as_ptr().drop_in_place();
                }
            }
            None => {}
        }
    }

    #[inline(always)]
    fn lock(&mut self) -> Result<(), Box<dyn Error>> {
        let mut err: bool = false;

        for holder in &mut self.holders {
            let tmp = unsafe { holder.1.as_mut() };

            match tmp.lock(holder.0 == self.last_lock_id, &mut self.page_pool) {
                Ok(_) => {}
                Err(_err) => {
                    err = true;
                }
            };
            if err {
                let _ = self.unlock();
                return Result::Err("Failed to lock".into());
            }
        }
        Result::Ok(())
    }

    #[inline(always)]
    fn unlock(&mut self) -> Result<(), Box<dyn Error>> {
        let mut err: bool = false;
        for holder in &mut self.holders {
            let tmp = unsafe { holder.1.as_mut() };
            match tmp.unlock(holder.0 == self.last_lock_id, &mut self.page_pool) {
                Ok(_) => {}
                Err(_err) => {
                    err = true;
                }
            }
        }
        if err {
            return Result::Err("Failed to unlock".into());
        }
        Result::Ok(())
    }

    pub unsafe fn get_page(&mut self, addr: u16) -> Option<NonNull<Page>> {
        let thing = self
            .page_pool
            .address_mapping
            .iter()
            .position(|val| *val == addr);
        if let Option::Some(addr) = thing {
            Option::Some(self.page_pool.pool.get_unchecked_mut(addr).into())
        } else {
            Option::None
        }
    }

    #[inline(never)]
    pub fn create_page(&mut self, addr: u16) -> Result<NonNull<Page>, Box<dyn Error>> {
        match self
            .page_pool
            .address_mapping
            .iter()
            .position(|val| *val >= addr)
        {
            Some(index) => {
                let val = unsafe { *self.page_pool.address_mapping.get_unchecked(index) };
                if val as u16 == addr {
                } else {
                    self.lock()?;
                    self.page_pool.address_mapping.insert(index, addr);
                    self.page_pool.pool.insert(index, Page::new());
                    self.unlock()?;
                }
            }
            None => {
                self.lock()?;
                self.page_pool.address_mapping.push(addr);
                self.page_pool.pool.push(Page::new());
                self.unlock()?;
            }
        }

        Result::Ok(
            self.page_pool
                .pool
                .get_mut(
                    self.page_pool
                        .address_mapping
                        .iter()
                        .position(|val| *val >= addr)
                        .unwrap(),
                )
                .unwrap()
                .into(),
        )
    }

    #[inline(always)]
    pub fn remove_all_pages(&mut self) -> Result<(), Box<dyn Error>> {
        self.lock()?;
        self.page_pool.address_mapping.clear();
        self.page_pool.pool.clear();
        self.unlock()?;
        Result::Ok(())
    }

    #[inline(never)]
    pub fn remove_page(&mut self, add: u16) -> Result<(), Box<dyn Error>> {
        let pos = self
            .page_pool
            .address_mapping
            .iter()
            .position(|i| *i == add);
        match pos {
            Some(add) => {
                self.lock()?;
                self.page_pool.address_mapping.remove(add);
                self.page_pool.pool.remove(add);
                self.unlock()?;
            }
            None => {}
        }
        Result::Ok(())
    }
}

//------------------------------------------------------------------------------------------------------

#[cfg(feature = "big_endian")]
#[macro_export]
macro_rules! get_mem_alligned {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32) -> $fn_type {
            (core::mem::transmute::<&u8, &$fn_type>(
                (*self.get_or_make_page(address).page_raw())
                    .get_unchecked_mut((address & 0xFFFF) as usize),
            ))
            .to_be()
        }
    };
}

#[cfg(not(feature = "big_endian"))]
#[macro_export]
macro_rules! get_mem_alligned {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32) -> $fn_type {
            unsafe {
                (core::mem::transmute::<&u8, &$fn_type>(
                    self.get_or_make_page(address)
                        .page
                        .get_unchecked_mut((address & 0xFFFF) as usize),
                ))
            }
        }
    };
}

#[cfg(feature = "big_endian")]
#[macro_export]
macro_rules! set_mem_alligned {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32, data: $fn_type) {
            (*core::mem::transmute::<&mut u8, &mut $fn_type>(
                (*self.get_or_make_page(address).page_raw())
                    .get_unchecked_mut((address & 0xFFFF) as usize),
            )) = data.to_be();
        }
    };
}

#[cfg(not(feature = "big_endian"))]
#[macro_export]
macro_rules! set_mem_alligned {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32, data: $fn_type) {
            unsafe {
                (*core::mem::transmute::<&mut u8, &mut $fn_type>(
                    self.get_or_make_page(address)
                        .page
                        .get_unchecked_mut((address & 0xFFFF) as usize),
                )) = data;
            }
        }
    };
}

#[cfg(feature = "big_endian")]
#[macro_export]
macro_rules! get_mem_alligned_o {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32) -> Option<$fn_type> {
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();

            match &mut self.get_page(address) {
                Option::Some(val) => {
                    return Option::Some(
                        (mem::transmute::<
                            &mut [u8; $crate::memory::page_pool::SEG_SIZE],
                            &mut [$fn_type;
                                     $crate::memory::page_pool::SEG_SIZE
                                         / mem::size_of::<$fn_type>()],
                        >(&mut *val.page_raw())[tmp])
                            .to_be(),
                    );
                }
                Option::None => {
                    return Option::None;
                }
            }
        }
    };
}

#[cfg(not(feature = "big_endian"))]
#[macro_export]
macro_rules! get_mem_alligned_o {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        fn $func_name(&'a mut self, address: u32) -> Option<$fn_type> {
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe {
                match &mut self.get_page(address) {
                    Option::Some(val) => {
                        return Option::Some(
                            mem::transmute::<
                                &mut [u8; $crate::memory::page_pool::SEG_SIZE],
                                &mut [$fn_type;
                                         $crate::memory::page_pool::SEG_SIZE
                                             / mem::size_of::<$fn_type>()],
                            >(&mut val.page)[tmp],
                        );
                    }
                    Option::None => {
                        return Option::None;
                    }
                }
            }
        }
    };
}

#[cfg(feature = "big_endian")]
#[macro_export]
macro_rules! set_mem_alligned_o {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32, data: $fn_type) -> Result<(), ()> {
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            match self.get_page(address) {
                Option::Some(mut val) => {
                    mem::transmute::<
                        &mut [u8; $crate::memory::page_pool::SEG_SIZE],
                        &mut [$fn_type;
                                 $crate::memory::page_pool::SEG_SIZE / mem::size_of::<$fn_type>()],
                    >(&mut *val.page_raw())[tmp] = data.to_be();

                    return Result::Ok(());
                }
                Option::None => {
                    return Result::Err(());
                }
            }
        }
    };
}

#[cfg(not(feature = "big_endian"))]
#[macro_export]
macro_rules! set_mem_alligned_o {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        fn $func_name(&mut self, address: u32, data: $fn_type) -> Result<(), ()> {
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            match self.get_page(address) {
                Option::Some(mut val) => {
                    unsafe {
                        mem::transmute::<
                            &mut [u8; $crate::memory::page_pool::SEG_SIZE],
                            &mut [$fn_type;
                                     $crate::memory::page_pool::SEG_SIZE
                                         / mem::size_of::<$fn_type>()],
                        >(&mut val.page)[tmp] = data;
                    }
                    return Result::Ok(());
                }
                Option::None => {
                    return Result::Err(());
                }
            }
        }
    };
}

//------------------------------------------------------------------------------------------------------

pub trait PagedMemoryInterface<'a>: PagedMemoryImpl
{
    type Page: PageImpl;


    unsafe fn get_or_make_page(&'a mut self, page: u32) -> Self::Page; //&mut Page;
    unsafe fn get_page(&'a mut self, page: u32) -> Option<Self::Page>; //Option<&mut Page>;

    unsafe fn copy_into_raw<T: Copy>(&'a mut self, address: u32, data: &[T]) {
        let size: usize = data.len() * mem::size_of::<T>();

        let data = core::slice::from_raw_parts(std::mem::transmute(data.as_ptr()), size);
        self.copy_into(address, data, 0, size);
    }

    unsafe fn get_or_make_mut_ptr_to_address(&'a mut self, address: u32) -> *mut u8 {
        &mut (*self.get_or_make_page(address).page_raw())[(address & 0xFFFF) as usize]
    }

    unsafe fn copy_into(&'a mut self, address: u32, data: &[u8], start: usize, end: usize) {
        let mut id = start;

        let mut tmp: Option<Self::Page> = Option::None;
        let ptr = self as *mut Self;

        for im in address..address + (end - start) as u32 {
            if im & 0xFFFF == 0 {
                tmp = Option::None;
            }
            match &mut tmp {
                None => {
                    let page = (*ptr).get_or_make_page(im);
                    tmp = Option::Some(page);
                }
                _ => {}
            }
            match &mut tmp {
                Some(val) => {
                    (*val.page_raw())[(im & 0xFFFF) as usize] = data[id];
                }
                None => panic!(),
            }
            id += 1;
        }
    }
}

/// # Safety
/// 
/// All methods in this trait are accessor methods to `PagedMemoryInterface`.
/// Since `PagedMemoryInterface` is a shared between threads any data accessed to written through these methods can cause race conditions
pub unsafe trait MemoryDefaultAccess<'a, P>
where
    P: PageImpl,
    Self: PagedMemoryInterface<'a>,
{
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

//------------------------------------------------------------------------------------------------------
