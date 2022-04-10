use std::{mem, sync::{Mutex, Arc, Weak, MutexGuard}, error::Error, cell::UnsafeCell, ops::{DerefMut, Deref}, borrow::BorrowMut, pin::Pin, collections::LinkedList, ptr::NonNull, fmt::Debug};

const SEG_SIZE:usize = 0x10000;
//stupid workaround
const INIT: Option<&'static mut Page> = None;


pub trait PagePoolHolder{
    fn lock(&mut self, page_pool: &mut PagePool) -> Result<(), Box<dyn Error>>;
    fn unlock(&mut self, page_pool: &mut PagePool) -> Result<(), Box<dyn Error>>;
}

pub struct PagePoolRef<T: PagePoolHolder + Send + Sync>{
    inner:  NonNull<T>,
    page_pool: Arc<Mutex<PagePoolController>>,
    id: usize,
}

unsafe impl<T: PagePoolHolder + Send + Sync> Send for PagePoolRef<T>{
    
}

unsafe impl<T: PagePoolHolder + Send + Sync> Sync for PagePoolRef<T>{
    
}

impl<T: PagePoolHolder + Send + Sync> Drop for PagePoolRef<T>{
    fn drop(&mut self) {
        log::warn!("Dropping PagePoolRef: {:p}", self);
        self.page_pool.lock().as_mut().unwrap().remove_holder(self.id);
    }
}

impl<T: PagePoolHolder + Send + Sync> PagePoolRef<T>{
    fn get_inner_mut<'a>(&'a mut self) -> &'a mut T{
        unsafe{self.inner.as_mut()}
    }

    fn get_inner<'a>(&'a self) -> &'a T{
        unsafe{self.inner.as_ref()}
    }

    fn get_page_pool(&self) -> Arc<Mutex<PagePoolController>>{
        self.page_pool.to_owned()
    } 
}

impl Default for PagePoolRef<Memory>{
    fn default() -> Self {
        Memory::new()
    }
}

impl<T: PagePoolHolder + Send + Sync> Deref for PagePoolRef<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_inner()
    }
}

impl<T: PagePoolHolder + Send + Sync> DerefMut for PagePoolRef<T>{

    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_inner_mut()
    }
}

#[derive(Default)]
pub struct PagePool{
    pub pool: Vec<Page>,
    pub address_mapping: Vec<u16>,
}

pub (crate) struct PagePoolController{
    page_pool: PagePool,
    holders: Vec<(usize, NonNull<dyn PagePoolHolder + Send + Sync>)>,
    myself: Weak<Mutex<PagePoolController>>,
}

impl Debug for PagePoolController{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PagePoolController").field("holders", &self.holders).field("myself", &self.myself).finish()
    }
}

unsafe impl Send for PagePoolController{

}
unsafe impl Sync for PagePoolController{
    
}

impl PagePoolController{

    fn new() -> Arc<Mutex<Self>>{
        let arc;
        unsafe{
            let tmp = mem::MaybeUninit::<PagePoolController>::zeroed();
            arc = Arc::new(Mutex::new(tmp.assume_init()));
            let weak = Arc::downgrade(&arc);
            
            match arc.lock().as_mut(){
                Ok(val) => {
                    let test = val.deref_mut() as *mut PagePoolController;
                    test.write(PagePoolController { 
                        page_pool: PagePool::default(), 
                        holders: Vec::new(), 
                        myself: weak 
                    });
                },
                Err(_err) => {
                    panic!();
                },
            }


        }
        arc
    }

    fn add_holder<T: PagePoolHolder + Send + Sync + 'static>(&mut self, holder: T) -> PagePoolRef<T>{
        let mut id: usize = 0;

        for holder in & self.holders{
            if holder.0 >= id{
                id = holder.0 + 1;
            }
        }

        let test = Box::new(holder);
        let ptr = NonNull::new(Box::into_raw(test)).unwrap();

        self.holders.push((id, ptr));

        
        PagePoolRef { 
            inner: ptr,
            page_pool: self.myself.upgrade().unwrap(), 
            id: id
        }
    }

    fn remove_holder(&mut self, id: usize){
        let index = self.holders.iter().position(|i| {
            id == i.0
        });
        //self.holders.iter_mut().f
        match index{
            Some(index) => {
                let item = self.holders.remove(index);
                let item = item.1;
                unsafe{
                    item.as_ptr().drop_in_place();   
                }
            },
            None => {}
        }
    }

    #[inline(always)]
    fn lock(&mut self) -> Result<(), Box<dyn Error>>{

        let mut err: bool = false;

        for holder in &mut self.holders{

            let tmp = unsafe{holder.1.as_mut()};
            
            match tmp.lock(&mut self.page_pool) {
                Ok(_) => {},
                Err(_err) => {
                    err = true;
                },
            };
            if err{
                let _ = self.unlock();
                return Result::Err("Failed to lock".into());
            }
        }
        Result::Ok(())
    }

    #[inline(always)]
    fn unlock(&mut self) -> Result<(), Box<dyn Error>>{
        let mut err: bool = false;
        for holder in &mut self.holders{

            let tmp = unsafe{holder.1.as_mut()};
            match tmp.unlock(&mut self.page_pool){
                Ok(_) => {},
                Err(_err) => {
                    err = true;
                }
            }
        }
        if err{
            return Result::Err("Failed to unlock".into());
        }
        Result::Ok(())
    }

    #[inline(always)]
    fn create_page(&mut self, addr: u16) -> Result<&mut Page, Box<dyn Error>>{

        self.lock()?;

        match self.page_pool.address_mapping.iter().position(|val|  {*val >= addr}) {
            Some(index) => {
                let val = unsafe{*self.page_pool.address_mapping.get_unchecked(index)};
                if val as u16 == addr{
                    
                }else{
                    self.page_pool.address_mapping.insert(index, addr);
                    self.page_pool.pool.insert(index, Page::new());
                }
            },
            None => {
                self.page_pool.address_mapping.push(addr);
                self.page_pool.pool.push(Page::new());
            },
        }

        self.unlock()?;
        Result::Ok(self.page_pool.pool.get_mut(self.page_pool.address_mapping.iter().position(|val|  {*val >= addr}).unwrap()).unwrap())
    }

    #[inline(always)]
    fn remove_all_pages(&mut self) -> Result<(), Box<dyn Error>>{
        self.lock()?;
        self.page_pool.address_mapping.clear();
        self.page_pool.pool.clear();
        self.unlock()?;
        Result::Ok(())
    }

    #[inline(always)]
    fn remove_page(&mut self, add: u16) -> Result<(), Box<dyn Error>>{
        
        let pos = self.page_pool.address_mapping.iter().position(|i| {
            *i == add
        });
        match pos {
            Some(add) => {
                self.lock()?;
                self.page_pool.address_mapping.remove(add);
                self.page_pool.pool.remove(add);
                self.unlock()?;
            },
            None => {

            },
        }
        Result::Ok(())
    }

}

impl PagePoolHolder for Memory{
    fn lock(&mut self, _page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        Result::Ok(())
    }

    fn unlock(&mut self, page_pool: &mut PagePool) -> Result<(), Box<dyn Error>> {
        
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

        Result::Ok(())
    }
}

pub struct Memory{
    pub(crate) page_pool: Option<Arc<Mutex<PagePoolController>>>,
    pub(crate) page_table: [Option<&'static mut Page>; SEG_SIZE],
}

impl Drop for Memory{
    fn drop(&mut self) {
        log::warn!("Droppping Memory: {:p}", self);
    }
}

macro_rules! get_mem_alligned {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32) -> $fn_type{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                *mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                    (&mut self.get_or_make_page(address).page).get_unchecked(tmp)
            }
        }
    };
}

macro_rules! set_mem_alligned {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32, data: $fn_type){
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                *mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                    (&mut self.get_or_make_page(address).page).get_unchecked_mut(tmp) = data;
            }
        }
    };
}

macro_rules! get_mem_alligned_o {
    ($func_name:ident, $fn_type:ty) => {
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32) -> Option<$fn_type>{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            unsafe{
                match &mut self.get_page(address){
                    Option::Some(val) => {
                        return Option::Some(
                            mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                            (&mut val.page)[tmp]);
                    }
                    Option::None => {
                        return Option::None;
                    }
                } 
                
            }
        }
    };
}

macro_rules! set_mem_alligned_o {
    // Arguments are module name and function name of function to tests bench
    ($func_name:ident, $fn_type:ty) => {
        // The macro will expand into the contents of this block.
        #[inline(always)]
        pub fn $func_name(&mut self, address: u32, data: $fn_type) -> Result<(), ()>{
            let tmp = (address & 0xFFFF) as usize / mem::size_of::<$fn_type>();
            match self.get_page(address){
                Option::Some(val) => {
                    unsafe{
                        mem::transmute::<&mut[u8; SEG_SIZE], &mut[$fn_type; SEG_SIZE / mem::size_of::<$fn_type>()]>
                            (&mut val.page)[tmp] = data;
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

#[allow(dead_code)]
impl Memory{

    pub fn new() -> PagePoolRef<Self>{
        let controller = PagePoolController::new();
        let mut lock = controller.lock();
        match lock.as_mut(){
            Ok(lock) => {
                let mem = Memory{
                    page_pool: Option::None,
                    page_table: [INIT; SEG_SIZE]
                };
                let mut mem = lock.add_holder(mem);
                let pool = mem.get_page_pool();
                mem.deref_mut().page_pool = Option::Some(pool);
                return mem;
            }
            Err(_err) => todo!(),
        }
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
                    
                    match &self.page_pool{
                        Some(val) => {
                            let mut val = val.lock().unwrap();
                            let val = val.borrow_mut();
                            let val = val.create_page(addr as u16);
                            //the value is already in memory but because I made this horrific system it gets optimized (rightfully so) so unless
                            //i dont set the value it will break and assume its empty
                            match val{
                                Ok(ok) => {
                                    *p = Option::Some(unsafe{mem::transmute(ok)});
                                },
                                Err(_) => {},
                            }
                        },
                        None => todo!(),
                    }

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
                let _ = val.lock().unwrap().borrow_mut().remove_page((address >> 16) as u16);
                //the values should already be set but this forces rust to 'relize' these values have actually been modified
                self.page_table[(address >> 16) as usize] = Option::None;
            },
            None => todo!(),
        }
    }
    pub fn unload_all_pages(&mut self) {
        match &self.page_pool{
            Some(val) => {
                let _ = val.lock().unwrap().borrow_mut().remove_all_pages();
                //the values should already be set but this forces rust to 'relize' these values have actually been modified
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

pub struct Page{
    page: [u8; SEG_SIZE],
}

impl Page{
    fn new() -> Self{
        Page{
            page: [0xdf; SEG_SIZE]
        }
    }
}