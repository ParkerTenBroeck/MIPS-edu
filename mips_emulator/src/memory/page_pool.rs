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
    fn init_page_pool_memory(&mut self, _notifier: Arc<Mutex<PagePoolController>>) {}
    fn get_notifier(&mut self) -> Option<&mut Arc<Mutex<PagePoolController>>>;
    fn lock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>>;
    fn unlock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>>;

    fn try_lock(
        &mut self,
        initiator: bool,
        page_pool: &PagePool,
    ) -> Result<(), TryLockError<Box<dyn Error>>>;
    fn try_unlock(&mut self, initiator: bool, page_pool: &PagePool) -> Result<(), Box<dyn Error>>;
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
    _guard: MutexGuard<'a, PagePoolController>,
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
}

unsafe impl Send for SharedPagePool {}

unsafe impl Sync for SharedPagePool {}

impl SharedPagePool {
    pub fn lock(&self) -> SharedPagePoolGuard {
        let controller = self.page_pool.lock().unwrap();

        SharedPagePoolGuard { guard: controller }
    }

    pub fn try_lock(&self) -> Result<SharedPagePoolGuard, TryLockError<Box<dyn Error>>> {
        match self.page_pool.try_lock() {
            Ok(controller) => Ok(SharedPagePoolGuard { guard: controller }),
            Err(err) => match err {
                std::sync::TryLockError::Poisoned(err) => {
                    Err(TryLockError::Error(err.to_string().into()))
                }
                std::sync::TryLockError::WouldBlock => Result::Err(TryLockError::WouldBlock),
            },
        }
    }

    pub fn clone_page_pool_mutex(&self) -> Arc<Mutex<PagePoolController>> {
        self.page_pool.clone()
    }

    pub fn new_controller_guard<T>(
        guard: MutexGuard<'_, PagePoolController>,
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
    holders: Vec<NonNull<dyn PagedMemoryImpl>>,
    myself: Option<Weak<Mutex<PagePoolController>>>,
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

fn addr_of_trait_object_nonnull_ptr(ptr: NonNull<dyn PagedMemoryImpl>) -> NonZeroU64 {
    let tmp = ptr.as_ptr().cast::<()>() as u64;
    NonZeroU64::new(tmp).unwrap()
}

fn addr_of_trait_object(ptr: &dyn PagedMemoryImpl) -> NonZeroU64 {
    let tmp = (ptr as *const dyn PagedMemoryImpl).cast::<()>() as u64;
    NonZeroU64::new(tmp).unwrap()
}

pub enum TryLockError<T> {
    Error(T),
    WouldBlock,
}

impl<T> From<std::sync::TryLockError<T>> for TryLockError<Box<dyn Error>> {
    fn from(err: std::sync::TryLockError<T>) -> Self {
        match err {
            std::sync::TryLockError::Poisoned(err) => Self::Error(err.to_string().into()),
            std::sync::TryLockError::WouldBlock => Self::WouldBlock,
        }
    }
}

impl PagePoolController {
    pub fn new() -> Arc<Mutex<Self>> {
        let arc = Arc::new(Mutex::new(PagePoolController {
            page_pool: PagePool::default(),
            holders: Vec::new(),
            myself: None,
        }));

        match arc.lock().as_mut() {
            Ok(val) => {
                val.myself = Some(Arc::downgrade(&arc));
            }
            Err(_err) => {
                panic!();
            }
        }
        arc
    }

    pub fn add_holder<T: PagedMemoryImpl + 'static>(
        &mut self,
        paged_memory: Box<T>,
    ) -> SharedPagePoolMemory<T> {
        let mut ptr = NonNull::new(Box::into_raw(paged_memory)).unwrap();

        self.holders.push(ptr);

        let shared = SharedPagePoolMemory {
            inner: ptr,
            page_pool: self.myself.as_mut().unwrap().upgrade().unwrap(),
        };
        unsafe { ptr.as_mut() }
            .init_page_pool_memory(self.myself.as_mut().unwrap().upgrade().unwrap());
        shared
    }

    pub fn remove_holder(&mut self, id: NonZeroU64) {
        let index = self
            .holders
            .iter()
            .position(|ptr| addr_of_trait_object_nonnull_ptr(*ptr) == id);
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

    fn try_lock(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
    ) -> Result<(), TryLockError<Box<dyn Error>>> {
        let mut index = 0;
        let (unlock_index, err) = loop {
            let mut holder = self.holders[index];

            let initiator =
                addr_of_trait_object_nonnull_ptr(holder) == addr_of_trait_object(requester);

            match {
                if initiator {
                    requester.try_lock(initiator, &self.page_pool)
                } else {
                    unsafe { holder.as_mut() }.try_lock(initiator, &self.page_pool)
                }
            } {
                Ok(_) => {}
                Err(err) => {
                    if index == 0 {
                        break (None, Some(err));
                    } else {
                        break (Some(index - 1), Some(err));
                    }
                }
            };
            if index == self.holders.len() {
                break (None, None);
            }
            index += 1;
        };

        if let Some(unlock_index) = unlock_index {
            for i in 0..unlock_index {
                let mut holder = self.holders[i];
                match unsafe { holder.as_mut() }.try_unlock(
                    addr_of_trait_object_nonnull_ptr(holder) == addr_of_trait_object(requester),
                    &self.page_pool,
                ) {
                    Ok(_) => {}
                    Err(_err) => {
                        //TODO put error somewhere
                    }
                };
            }

            if let Some(err) = err {
                Result::Err(err)
            } else {
                Result::Err(TryLockError::WouldBlock)
            }
        } else {
            Result::Ok(())
        }
    }

    fn try_unlock(&mut self, requester: &mut dyn PagedMemoryImpl) -> Result<(), Box<dyn Error>> {
        let mut vec = Vec::new();

        for holder in &mut self.holders {
            match unsafe { holder.as_mut() }.try_unlock(
                addr_of_trait_object_nonnull_ptr(*holder) == addr_of_trait_object(requester),
                &self.page_pool,
            ) {
                Ok(_) => {}
                Err(err) => vec.push(err),
            }
        }

        if vec.is_empty() {
            Result::Ok(())
        } else {
            Result::Err(format!("{:#?}", vec).into())
        }
    }

    fn lock(&mut self, requester: &mut dyn PagedMemoryImpl) -> Result<(), Box<dyn Error>> {
        let mut err: bool = false;

        for holder in &mut self.holders {
            let mut tmp = unsafe { holder.as_mut() };

            let initiator =
                addr_of_trait_object_nonnull_ptr(*holder) == addr_of_trait_object(requester);
            if initiator {
                tmp = requester;
            }

            //if Some(addr_of_trait_object_nonnull_ptr(*holder)) != addr_of_trait_object(requester){
            match tmp.lock(initiator, &self.page_pool) {
                Ok(_) => {}
                Err(_err) => {
                    err = true;
                }
            };
            //}
            if err {
                let _ = self.unlock(requester);
                return Result::Err("Failed to lock".into());
            }
        }
        Result::Ok(())
    }

    #[inline(always)]
    fn unlock(&mut self, requester: &mut dyn PagedMemoryImpl) -> Result<(), Box<dyn Error>> {
        let mut err: bool = false;
        for holder in &mut self.holders {
            let tmp = unsafe { holder.as_mut() };
            match tmp.unlock(
                addr_of_trait_object_nonnull_ptr(*holder) == addr_of_trait_object(requester),
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
    pub unsafe fn create_page(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
        addr: u16,
    ) -> Result<NonNull<Page>, Box<dyn Error>> {
        match self
            .page_pool
            .address_mapping
            .iter()
            .position(|val| *val >= addr)
        {
            Some(index) => {
                let val = unsafe { *self.page_pool.address_mapping.get_unchecked(index) };
                if val == addr {
                } else {
                    self.lock(requester)?;
                    self.page_pool.address_mapping.insert(index, addr);
                    self.page_pool.pool.insert(index, Page::new());
                    self.unlock(requester)?;
                }
            }
            None => {
                self.lock(requester)?;
                self.page_pool.address_mapping.push(addr);
                self.page_pool.pool.push(Page::new());
                self.unlock(requester)?;
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

    /// # Safety
    ///
    /// The returned pointer must not outlive this `SharedPagePool`.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls lock
    #[inline(never)]
    pub unsafe fn try_create_page(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
        addr: u16,
    ) -> Result<NonNull<Page>, TryLockError<Box<dyn Error>>> {
        impl From<Box<dyn Error>> for TryLockError<Box<dyn Error>> {
            fn from(err: Box<dyn Error>) -> Self {
                TryLockError::Error(err)
            }
        }

        match self
            .page_pool
            .address_mapping
            .iter()
            .position(|val| *val >= addr)
        {
            Some(index) => {
                let val = unsafe { *self.page_pool.address_mapping.get_unchecked(index) };
                if val == addr {
                } else {
                    self.try_lock(requester)?;
                    self.page_pool.address_mapping.insert(index, addr);
                    self.page_pool.pool.insert(index, Page::new());
                    self.try_unlock(requester)?;
                }
            }
            None => {
                self.try_lock(requester)?;
                self.page_pool.address_mapping.push(addr);
                self.page_pool.pool.push(Page::new());
                self.try_unlock(requester)?;
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

    pub fn remove_all_pages(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
    ) -> Result<(), Box<dyn Error>> {
        self.lock(requester)?;
        self.page_pool.address_mapping.clear();
        self.page_pool.pool.clear();
        self.unlock(requester)?;
        Result::Ok(())
    }

    pub fn try_remove_all_pages(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
    ) -> Result<(), TryLockError<Box<dyn Error>>> {
        self.try_lock(requester)?;
        self.page_pool.address_mapping.clear();
        self.page_pool.pool.clear();
        self.try_lock(requester)?;
        Result::Ok(())
    }

    #[inline(never)]
    pub fn remove_page(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
        add: u16,
    ) -> Result<(), Box<dyn Error>> {
        let pos = self
            .page_pool
            .address_mapping
            .iter()
            .position(|i| *i == add);
        if let Some(add) = pos {
            self.lock(requester)?;
            self.page_pool.address_mapping.remove(add);
            self.page_pool.pool.remove(add);
            self.unlock(requester)?;
        }
        Result::Ok(())
    }

    #[inline(never)]
    pub fn try_remove_page(
        &mut self,
        requester: &mut dyn PagedMemoryImpl,
        add: u16,
    ) -> Result<(), TryLockError<Box<dyn Error>>> {
        let pos = self
            .page_pool
            .address_mapping
            .iter()
            .position(|i| *i == add);
        if let Some(add) = pos {
            self.try_lock(requester)?;
            self.page_pool.address_mapping.remove(add);
            self.page_pool.pool.remove(add);
            self.try_unlock(requester)?;
        }
        Result::Ok(())
    }
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
    /// The returned pointer must not outlive self.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls the lock method on this trait objects `PagedMemoryImpl`
    unsafe fn try_get_or_make_page(
        &'a mut self,
        address: u32,
    ) -> Result<Self::Page, TryLockError<Box<dyn Error>>>;

    /// # Safety
    ///
    /// The returned pointer must not outlive self.
    ///
    /// The returned pointer must also be destroyed after this `SharedPagePool` calls the lock method on this trait objects `PagedMemoryImpl`
    unsafe fn try_get_page(
        &'a mut self,
        address: u32,
    ) -> Result<Option<Self::Page>, TryLockError<Box<dyn Error>>>;

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

//------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! get_mem_alligned_be {
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

#[macro_export]
macro_rules! get_mem_alligned_le {
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
            .to_le()
        }
    };
}

#[macro_export]
macro_rules! set_mem_alligned_be {
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

#[macro_export]
macro_rules! set_mem_alligned_le {
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
            )) = data.to_le();
        }
    };
}

#[macro_export]
macro_rules! get_mem_alligned_o_be {
    ($func_name:ident, $fn_type:ty) => {
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data returned is from a `SharedPagePool`, this data is shared between threads and does not protect against race conditions
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32) -> Option<$fn_type> {
            match &mut self.get_page(address) {
                Option::Some(val) => {
                    return Option::Some(
                        (core::mem::transmute::<&u8, &$fn_type>(
                            (*val.page_raw()).get_unchecked_mut((address & 0xFFFF) as usize),
                        ))
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

#[macro_export]
macro_rules! get_mem_alligned_o_le {
    ($func_name:ident, $fn_type:ty) => {
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data returned is from a `SharedPagePool`, this data is shared between threads and does not protect against race conditions
        #[inline(always)]
        unsafe fn $func_name(&'a mut self, address: u32) -> Option<$fn_type> {
            match &mut self.get_page(address) {
                Option::Some(val) => {
                    return Option::Some(
                        (core::mem::transmute::<&u8, &$fn_type>(
                            (*val.page_raw()).get_unchecked_mut((address & 0xFFFF) as usize),
                        ))
                        .to_le(),
                    );
                }
                Option::None => {
                    return Option::None;
                }
            }
        }
    };
}

#[macro_export]
macro_rules! set_mem_alligned_o_be {
    ($func_name:ident, $fn_type:ty) => {
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data is written to this traits underlying `SharedPagePool`, this can cause race conditions
        #[inline(always)]
        #[allow(clippy::result_unit_err)]
        unsafe fn $func_name(&'a mut self, address: u32, data: $fn_type) -> Result<(), ()> {
            match &mut self.get_page(address) {
                Option::Some(val) => {
                    (*core::mem::transmute::<&mut u8, &mut $fn_type>(
                        (*val.page_raw()).get_unchecked_mut((address & 0xFFFF) as usize),
                    )) = data.to_be();

                    return Result::Ok(());
                }
                Option::None => {
                    return Result::Err(());
                }
            }
        }
    };
}

#[macro_export]
macro_rules! set_mem_alligned_o_le {
    ($func_name:ident, $fn_type:ty) => {
        /// # Safety
        ///
        /// address must be alligned to datas alignment
        ///
        /// The data is written to this traits underlying `SharedPagePool`, this can cause race conditions
        #[inline(always)]
        #[allow(clippy::result_unit_err)]
        unsafe fn $func_name(&'a mut self, address: u32, data: $fn_type) -> Result<(), ()> {
            match &mut self.get_page(address) {
                Option::Some(val) => {
                    (*core::mem::transmute::<&mut u8, &mut $fn_type>(
                        (*val.page_raw()).get_unchecked_mut((address & 0xFFFF) as usize),
                    )) = data.to_le();
                    return Result::Ok(());
                }
                Option::None => {
                    return Result::Err(());
                }
            }
        }
    };
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
    get_mem_alligned_be!(get_i64_alligned_be, i64);
    set_mem_alligned_be!(set_i64_alligned_be, i64);
    get_mem_alligned_be!(get_u64_alligned_be, u64);
    set_mem_alligned_be!(set_u64_alligned_be, u64);

    get_mem_alligned_be!(get_i32_alligned_be, i32);
    set_mem_alligned_be!(set_i32_alligned_be, i32);
    get_mem_alligned_be!(get_u32_alligned_be, u32);
    set_mem_alligned_be!(set_u32_alligned_be, u32);

    get_mem_alligned_be!(get_i16_alligned_be, i16);
    set_mem_alligned_be!(set_i16_alligned_be, i16);
    get_mem_alligned_be!(get_u16_alligned_be, u16);
    set_mem_alligned_be!(set_u16_alligned_be, u16);

    get_mem_alligned_be!(get_i8_be, i8);
    set_mem_alligned_be!(set_i8_be, i8);
    get_mem_alligned_be!(get_u8_be, u8);
    set_mem_alligned_be!(set_u8_be, u8);

    get_mem_alligned_o_be!(get_i64_alligned_o_be, i64);
    set_mem_alligned_o_be!(set_i64_alligned_o_be, i64);
    get_mem_alligned_o_be!(get_u64_alligned_o_be, u64);
    set_mem_alligned_o_be!(set_u64_alligned_o_be, u64);

    get_mem_alligned_o_be!(get_i32_alligned_o_be, i32);
    set_mem_alligned_o_be!(set_i32_alligned_o_be, i32);
    get_mem_alligned_o_be!(get_u32_alligned_o_be, u32);
    set_mem_alligned_o_be!(set_u32_alligned_o_be, u32);

    get_mem_alligned_o_be!(get_i16_alligned_o_be, i16);
    set_mem_alligned_o_be!(set_i16_alligned_o_be, i16);
    get_mem_alligned_o_be!(get_u16_alligned_o_be, u16);
    set_mem_alligned_o_be!(set_u16_alligned_o_be, u16);

    get_mem_alligned_o_be!(get_i8_o_be, i8);
    set_mem_alligned_o_be!(set_i8_o_be, i8);
    get_mem_alligned_o_be!(get_u8_o_be, u8);
    set_mem_alligned_o_be!(set_u8_o_be, u8);

    get_mem_alligned_le!(get_i64_alligned_le, i64);
    set_mem_alligned_le!(set_i64_alligned_le, i64);
    get_mem_alligned_le!(get_u64_alligned_le, u64);
    set_mem_alligned_le!(set_u64_alligned_le, u64);

    get_mem_alligned_le!(get_i32_alligned_le, i32);
    set_mem_alligned_le!(set_i32_alligned_le, i32);
    get_mem_alligned_le!(get_u32_alligned_le, u32);
    set_mem_alligned_le!(set_u32_alligned_le, u32);

    get_mem_alligned_le!(get_i16_alligned_le, i16);
    set_mem_alligned_le!(set_i16_alligned_le, i16);
    get_mem_alligned_le!(get_u16_alligned_le, u16);
    set_mem_alligned_le!(set_u16_alligned_le, u16);

    get_mem_alligned_le!(get_i8_le, i8);
    set_mem_alligned_le!(set_i8_le, i8);
    get_mem_alligned_le!(get_u8_le, u8);
    set_mem_alligned_le!(set_u8_le, u8);

    get_mem_alligned_o_le!(get_i64_alligned_o_le, i64);
    set_mem_alligned_o_le!(set_i64_alligned_o_le, i64);
    get_mem_alligned_o_le!(get_u64_alligned_o_le, u64);
    set_mem_alligned_o_le!(set_u64_alligned_o_le, u64);

    get_mem_alligned_o_le!(get_i32_alligned_o_le, i32);
    set_mem_alligned_o_le!(set_i32_alligned_o_le, i32);
    get_mem_alligned_o_le!(get_u32_alligned_o_le, u32);
    set_mem_alligned_o_le!(set_u32_alligned_o_le, u32);

    get_mem_alligned_o_le!(get_i16_alligned_o_le, i16);
    set_mem_alligned_o_le!(set_i16_alligned_o_le, i16);
    get_mem_alligned_o_le!(get_u16_alligned_o_le, u16);
    set_mem_alligned_o_le!(set_u16_alligned_o_le, u16);

    get_mem_alligned_o_le!(get_i8_o_le, i8);
    set_mem_alligned_o_le!(set_i8_o_le, i8);
    get_mem_alligned_o_le!(get_u8_o_le, u8);
    set_mem_alligned_o_le!(set_u8_o_le, u8);
}

//------------------------------------------------------------------------------------------------------
