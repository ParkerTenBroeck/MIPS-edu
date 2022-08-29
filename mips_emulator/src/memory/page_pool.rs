use std::{
    error::Error,
    fmt::Debug,
    mem,
    num::NonZeroU64,
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
    /// # Safety
    ///
    /// The returned pointer must not outlive self
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

pub trait PagedMemoryImpl: Sync + Send {
    fn init_notifier(&mut self, _notifier: SharedPagePool) {}
    fn get_notifier(&mut self) -> Option<&mut SharedPagePool>;
    fn lock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>>;
    fn unlock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>>;
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
    _guard: SharedPagePoolGuard<'a>,
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

pub struct SharedPagePool {
    page_pool: Arc<Mutex<PagePoolController>>,
    id: Option<NonZeroU64>,
}

unsafe impl Send for SharedPagePool {}

unsafe impl Sync for SharedPagePool {}

impl SharedPagePool {
    pub fn lock(&mut self) -> MutexGuard<PagePoolController> {
        let mut guard = self.page_pool.lock().unwrap();
        guard.last_lock_id = self.id;
        guard
    }
}

impl SharedPagePool {
    pub unsafe fn get_raw_pool(&self) -> &Arc<Mutex<PagePoolController>> {
        &self.page_pool
    }

    pub fn get_page_pool(&self) -> SharedPagePoolGuard {
        let mut controller = self.page_pool.lock().unwrap();
        controller.last_lock_id = self.id;

        SharedPagePoolGuard { guard: controller }
    }

    pub fn clone_page_pool_mutex(&self) -> Arc<Mutex<PagePoolController>> {
        self.page_pool.clone()
    }

    pub fn new_controller_guard<T>(
        guard: SharedPagePoolGuard<'_>,
        data: T,
    ) -> ControllerGuard<'_, T> {
        ControllerGuard {
            _guard: guard,
            data,
        }
    }
}

//------------------------------------------------------------------------------------------------------

pub struct SharedPagePoolGuard<'a> {
    guard: MutexGuard<'a, PagePoolController>,
}

impl<'a> SharedPagePoolGuard<'a> {
    pub unsafe fn from_raw(
        mut guard: MutexGuard<'a, PagePoolController>,
        notifier: &SharedPagePool,
    ) -> Self {
        guard.last_lock_id = notifier.id;
        Self { guard }
    }
}

impl<'a> Drop for SharedPagePoolGuard<'a> {
    fn drop(&mut self) {
        self.guard.last_lock_id = None;
    }
}
impl<'a> Deref for SharedPagePoolGuard<'a> {
    type Target = MutexGuard<'a, PagePoolController>;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}
impl<'a> DerefMut for SharedPagePoolGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

//------------------------------------------------------------------------------------------------------

pub struct SharedPagePoolMemory<T: PagedMemoryImpl> {
    inner: NonNull<T>,
    page_pool: Arc<Mutex<PagePoolController>>,
}

unsafe impl<T: PagedMemoryImpl> Send for SharedPagePoolMemory<T> {}

unsafe impl<T: PagedMemoryImpl> Sync for SharedPagePoolMemory<T> {}

impl<T: PagedMemoryImpl> Drop for SharedPagePoolMemory<T> {
    fn drop(&mut self) {
        let addr = self.inner.as_ptr() as u64;
        self.page_pool
            .lock()
            .unwrap()
            .remove_holder(NonZeroU64::new(addr).unwrap());
    }
}

impl<T: PagedMemoryImpl> SharedPagePoolMemory<T> {
    fn get_inner_mut(&mut self) -> &mut T {
        unsafe { self.inner.as_mut() }
    }

    fn get_inner(&self) -> &T {
        unsafe { self.inner.as_ref() }
    }
}

impl<T: PagedMemoryImpl + Send + Sync> Deref for SharedPagePoolMemory<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_inner()
    }
}

impl<T: PagedMemoryImpl + Send + Sync> DerefMut for SharedPagePoolMemory<T> {
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
    holders: Vec<NonNull<dyn PagedMemoryImpl + Send + Sync>>,
    myself: Weak<Mutex<PagePoolController>>,
    last_lock_id: Option<NonZeroU64>,
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

fn addr_of_trait_object(ptr: NonNull<dyn PagedMemoryImpl + Send + Sync>) -> NonZeroU64 {
    let tmp = ptr.as_ptr().cast::<()>() as u64;
    NonZeroU64::new(tmp).unwrap()
}

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
                        last_lock_id: None,
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
        paged_memory: Box<T>,
    ) -> SharedPagePoolMemory<T> {
        let mut ptr = NonNull::new(Box::into_raw(paged_memory)).unwrap();

        let id = ptr.as_ptr() as u64;
        let id = NonZeroU64::new(id);

        self.holders.push(ptr);

        let shared = SharedPagePoolMemory {
            inner: ptr,
            page_pool: self.myself.upgrade().unwrap(),
        };

        unsafe { ptr.as_mut() }.init_notifier(SharedPagePool {
            page_pool: self.myself.upgrade().unwrap(),
            id,
        });

        shared
    }

    pub fn remove_holder(&mut self, id: NonZeroU64) {
        let index = self
            .holders
            .iter()
            .position(|ptr| addr_of_trait_object(*ptr) == id);
        //self.holders.iter_mut().f
        if let Some(index) = index {
            let item = self.holders.remove(index);
            unsafe {
                item.as_ptr().drop_in_place();
            }
        } else {
            panic!()
        }
    }

    #[inline(always)]
    fn lock(&mut self) -> Result<(), Box<dyn Error>> {
        let mut err: bool = false;

        for holder in &mut self.holders {
            let tmp = unsafe { holder.as_mut() };

            match tmp.lock(
                Some(addr_of_trait_object(*holder)) == self.last_lock_id,
                &self.page_pool,
            ) {
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
            let tmp = unsafe { holder.as_mut() };
            match tmp.unlock(
                Some(addr_of_trait_object(*holder)) == self.last_lock_id,
                &self.page_pool,
            ) {
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

    /// # Safety
    ///
    /// The returned pointer must not outlive this `SharedPagePool`.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls lock
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

    /// # Safety
    ///
    /// The returned pointer must not outlive this `SharedPagePool`.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls lock
    #[inline(never)]
    pub unsafe fn create_page(&mut self, addr: u16) -> Result<NonNull<Page>, Box<dyn Error>> {
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
        if let Some(add) = pos {
            self.lock()?;
            self.page_pool.address_mapping.remove(add);
            self.page_pool.pool.remove(add);
            self.unlock()?;
        }
        Result::Ok(())
    }
}

//------------------------------------------------------------------------------------------------------

#[cfg(feature = "big_endian")]
#[macro_export]
macro_rules! get_mem_alligned {
    ($func_name:ident, $fn_type:ty) => {
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data returned is from a `SharedPagePool`, this data is shared between threads and does not protect against race conditions
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
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data is written to this traits underlying `SharedPagePool`, this can cause race conditions
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
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data returned is from a `SharedPagePool`, this data is shared between threads and does not protect against race conditions
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
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data is written to this traits underlying `SharedPagePool`, this can cause race conditions
        #[inline(always)]
        #[allow(clippy::result_unit_err)]
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

pub trait PagedMemoryInterface<'a>: PagedMemoryImpl {
    type Page: PageImpl;

    /// # Safety
    ///
    /// The returned pointer must not outlive self.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls the lock method on this trait objects `PagedMemoryImpl`
    unsafe fn get_or_make_page(&'a mut self, page: u32) -> Self::Page;

    /// # Safety
    ///
    /// The returned pointer must not outlive self.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls the lock method on this trait objects `PagedMemoryImpl`
    unsafe fn get_page(&'a mut self, page: u32) -> Option<Self::Page>;

    /// # Safety
    ///
    /// This data is directly written to this trait objects `SharedPagePool` starting at `address`.
    ///
    /// This can cause race conditions
    unsafe fn copy_into_raw<T: Copy>(&'a mut self, address: u32, data: &[T]) {
        let size: usize = data.len() * mem::size_of::<T>();

        let data = core::slice::from_raw_parts(std::mem::transmute(data.as_ptr()), size);
        self.copy_into(address, data, 0, size);
    }

    /// # Safety
    ///
    /// This data is directly written to this trait objects `SharedPagePool` starting at `address`.
    ///
    /// This can cause race conditions
    unsafe fn copy_into(&'a mut self, address: u32, data: &[u8], start: usize, end: usize) {
        let mut id = start;

        let mut tmp: Option<Self::Page> = Option::None;
        let ptr = self as *mut Self;

        for im in address..address + (end - start) as u32 {
            if im & 0xFFFF == 0 {
                tmp = Option::None;
            }
            if tmp.is_none() {
                let page = (*ptr).get_or_make_page(im);
                tmp = Option::Some(page);
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
