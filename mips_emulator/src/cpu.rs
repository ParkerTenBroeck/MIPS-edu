use core::panic;
use std::{
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::{atomic::AtomicUsize, Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use crate::memory::{
    emulator_memory::Memory,
    page_pool::{
        MemoryDefaultAccess, PagePoolController, PagedMemoryImpl, PagedMemoryInterface,
        SharedPagePoolMemory,
    },
};

//macros
//jump encoding
macro_rules! jump_immediate_address {
    ($expr:expr) => {
        ((($expr as u32) & 0b00000011111111111111111111111111) << 2)
    };
}

#[allow(unused)]
macro_rules! jump_immediate_offset {
    ($expr:expr) => {
        (($expr as i32) << 6) >> 4
    };
}

//immediate encoding
macro_rules! immediate_immediate_signed_extended {
    ($expr:expr) => {
        ((($expr as i32) << 16) >> 16) as u32
    };
}
macro_rules! immediate_immediate_zero_extended {
    ($expr:expr) => {
        (($expr as u32) & 0xFFFF)
    };
}

macro_rules! immediate_immediate_address {
    ($expr:expr) => {
        (($expr as i32) << 16) >> 14
    };
}

#[allow(unused)]
macro_rules! immediate_immediate_unsigned_hi {
    ($expr:expr) => {
        (($expr as u32) << 16)
    };
}

macro_rules! immediate_s {
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    };
}

macro_rules! immediate_t {
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    };
}

macro_rules! register_s {
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    };
}

macro_rules! register_t {
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    };
}

macro_rules! register_d {
    ($expr:expr) => {
        ((($expr as u32) >> 11) & 0b11111) as usize
    };
}

macro_rules! register_a {
    ($expr:expr) => {
        (($expr as u32) >> 6) & 0b11111
    };
}

//Co processor macros
// macro_rules! cop1_function {
//     ($expr:expr) => {
//         ($expr as u32) & 0b111111
//     };
// }

// macro_rules! cop1_fd {
//     ($expr:expr) => {
//         (($expr as u32) >> 6) & 0b11111
//     };
// }
// macro_rules! cop1_fs {
//     ($expr:expr) => {
//         (($expr as u32) >> 11) & 0b11111
//     };
// }
// macro_rules! cop1_ft {
//     ($expr:expr) => {
//         (($expr as u32) >> 16) & 0b11111
//     };
// }
// macro_rules! cop1_fmt {
//     ($expr:expr) => {
//         (($expr as u32) >> 21) & 0b11111
//     };
// }

//Macros
pub struct EmulatorInterface<T: CpuExternalHandler> {
    inner: Pin<Arc<(MipsCpu<T>, AtomicUsize)>>,
}

impl<T: CpuExternalHandler> Clone for EmulatorInterface<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: CpuExternalHandler> EmulatorInterface<T> {
    pub fn new(cpu: MipsCpu<T>) -> Self {
        Self {
            inner: Arc::pin((cpu, 0.into())),
        }
    }
    fn lock<R>(&mut self, fn_once: impl FnOnce(&mut Self) -> R) -> R {
        loop {
            let item = self.inner.1.load(std::sync::atomic::Ordering::Relaxed);
            if item < usize::MAX - 1
                && self
                    .inner
                    .1
                    .compare_exchange(
                        item,
                        item + 1,
                        std::sync::atomic::Ordering::Acquire,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                    .is_ok()
            {
                let ret = fn_once(self);
                self.inner
                    .1
                    .fetch_sub(1, std::sync::atomic::Ordering::Release);
                return ret;
            }
            std::hint::spin_loop()
        }
    }
    fn lock_mut<R>(&mut self, fn_once: impl FnOnce(&mut Self) -> R) -> R {
        loop {
            if self
                .inner
                .1
                .compare_exchange(
                    0,
                    usize::MAX,
                    std::sync::atomic::Ordering::Acquire,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                let ret = fn_once(self);
                if self
                    .inner
                    .1
                    .compare_exchange(
                        usize::MAX,
                        0,
                        std::sync::atomic::Ordering::Release,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return ret;
                } else {
                    panic!();
                }
            }
            std::hint::spin_loop()
        }
    }
    pub fn cpu_mut<R>(&mut self, fn_once: impl FnOnce(&mut MipsCpu<T>) -> R) -> R {
        self.lock_mut(|iner| unsafe {
            (*iner.raw_cpu_mut()).pause();
            let result = fn_once(&mut *iner.raw_cpu_mut());
            (*iner.raw_cpu_mut()).resume();
            result
        })
    }
    pub fn cpu_ref(&mut self, fn_once: impl FnOnce(&MipsCpu<T>)) {
        self.lock(|inner| unsafe {
            (*inner.raw_cpu_mut()).pause();
            fn_once(&*inner.raw_cpu());
            (*inner.raw_cpu_mut()).resume();
        });
    }
    pub fn start(
        &mut self,
        runner: impl FnOnce(Box<dyn FnOnce() + Sync + Send>),
    ) -> Result<(), &str> {
        self.lock_mut(|inner| unsafe {
            if (*inner.raw_cpu_mut()).is_running() {
                Result::Err("Cannot start, emulator is already running")
            } else {
                let mut cpy = inner.clone();
                runner(Box::new(move || (*cpy.raw_cpu_mut()).start_local()));
                Result::Ok(())
            }
        })
    }
    pub fn start_new_thread(&mut self) -> Result<(), &str> {
        self.lock_mut(|inner| unsafe {
            if (*inner.raw_cpu_mut()).is_running() {
                Result::Err("Cannot start, emulator is already running")
            } else {
                (*inner.raw_cpu_mut()).start_new_thread();
                Result::Ok(())
            }
        })
    }
    pub fn step(
        &mut self,
        runner: impl FnOnce(Box<dyn FnOnce() + Sync + Send>),
    ) -> Result<(), &str> {
        self.lock_mut(|inner| unsafe {
            if (*inner.raw_cpu_mut()).is_running() {
                Result::Err("Cannot step, emulator is already running")
            } else {
                let mut cpy = inner.clone();
                runner(Box::new(move || (*cpy.raw_cpu_mut()).step_local()));
                Result::Ok(())
            }
        })
    }
    pub fn step_new_thread(&mut self) -> Result<(), &str> {
        self.lock_mut(|inner| unsafe {
            if (*inner.raw_cpu_mut()).is_running() {
                Result::Err("Cannot start, emulator is already running")
            } else {
                (*inner.raw_cpu_mut()).step_new_thread();
                Result::Ok(())
            }
        })
    }
    pub fn stop(&mut self) -> Result<(), &str> {
        self.lock_mut(|inner| unsafe {
            if (*inner.raw_cpu_mut()).is_running() {
                (*inner.raw_cpu_mut()).stop_and_wait();
                Result::Ok(())
            } else {
                Result::Err("Emulator is already stopped")
            }
        })
    }
    pub fn restart(&mut self) -> Result<(), &str> {
        self.lock_mut(|inner| unsafe {
            if (*inner.raw_cpu_mut()).is_running() {
                Result::Err("Cannot reset while emulator is running")
            } else {
                (*inner.raw_cpu_mut()).reset();
                Result::Ok(())
            }
        })
    }
    unsafe fn raw_cpu_mut(&mut self) -> *mut MipsCpu<T> {
        &self.inner.0 as *const MipsCpu<T> as *mut MipsCpu<T>
    }

    /// # Safety
    ///
    /// This method ensures that the no other instance of `EmulatorInterface<T>` that holds this `*mut MipsCpu<T>` can mutate or access the pointer until `fn_once` returns.
    ///
    /// However this method does not ensure that `*mut MipsCpu<T>` isn't being accessed in another thread i.e the emulator is running in a separate thread.
    ///
    /// Be carful as mutating or dereferencing the pointer can still cause race conditions.
    ///
    /// Do not move the pointer outside of `fn_once`.
    pub unsafe fn lock_raw_cpu_mut<R>(&mut self, fn_once: impl FnOnce(*mut MipsCpu<T>) -> R) -> R {
        self.lock_mut(|inner| fn_once(inner.raw_cpu_mut()))
    }

    /// # Safety
    ///
    /// There is no guarantee that the return value is not being accessed by other threads
    ///
    /// This can give rise to race conditions if dereferencing the pointer
    pub unsafe fn raw_cpu(&self) -> *const MipsCpu<T> {
        &self.inner.0 as *const MipsCpu<T>
    }

    /// # Safety
    ///
    /// There is no guarantee that the data being accessed is being written to or accessed by other threads.
    ///
    /// The value returned can be mutated at any moment and can cause race conditions.
    #[inline(always)]
    pub unsafe fn pc(&self) -> u32 {
        (*self.raw_cpu()).pc
    }

    /// # Safety
    ///
    /// There is no guarantee that the data being accessed is being written to or accessed by other threads.
    ///
    /// The value returned can be mutated at any moment and can cause race conditions.
    #[inline(always)]
    pub unsafe fn reg(&self) -> &[u32; 32] {
        &(*self.raw_cpu()).reg
    }

    /// # Safety
    ///
    /// There is no guarantee that the data being accessed is being written to or accessed by other threads.
    ///
    /// The value returned can be mutated at any moment and can cause race conditions.
    #[inline(always)]
    pub unsafe fn lo(&self) -> u32 {
        (*self.raw_cpu()).lo
    }

    /// # Safety
    ///
    /// There is no guarantee that the data being accessed is being written to or accessed by other threads.
    ///
    /// The value returned can be mutated at any moment and can cause race conditions.
    #[inline(always)]
    pub unsafe fn hi(&self) -> u32 {
        (*self.raw_cpu()).hi
    }
}

pub trait EmulatorPause: 'static {
    /// # Safety
    ///
    /// Must call resume after
    unsafe fn pause(&mut self);

    /// # Safety
    ///
    /// Must call resume after
    #[allow(clippy::result_unit_err)]
    unsafe fn try_pause(&mut self, iterations: usize) -> Result<(), ()>;

    /// # Safety
    ///
    /// Must call pause before
    unsafe fn resume(&mut self);
}
impl<T: CpuExternalHandler> EmulatorPause for EmulatorInterface<T> {
    unsafe fn pause(&mut self) {
        (*(self.raw_cpu() as *mut MipsCpu<T>)).pause()
    }
    unsafe fn resume(&mut self) {
        (*(self.raw_cpu() as *mut MipsCpu<T>)).resume()
    }

    unsafe fn try_pause(&mut self, iterations: usize) -> Result<(), ()> {
        let cpu = &mut (*(self.raw_cpu() as *mut MipsCpu<T>));

        cpu.paused
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        for _ in 0..iterations {
            core::ptr::write_volatile(&mut cpu.check, true);
            if cpu.is_paused() || !cpu.is_running() {
                return Result::Ok(());
            }
        }
        self.resume();
        Result::Err(())
    }
}

//-------------------------------------------------------- co processors

#[derive(Default)]
pub struct CP0 {
    _registers: [u32; 32],
}

#[repr(C)]
pub union CP1Reg {
    pub single: [f32; 32],
    pub double: [f64; 16],
}

pub struct CP1 {
    _registers: CP1Reg,
}

impl Default for CP1 {
    fn default() -> Self {
        CP1 {
            _registers: CP1Reg { single: [0.0; 32] },
        }
    }
}

pub trait Debugger<T: CpuExternalHandler>: 'static + Sync + Send {
    fn detach(&mut self, cpu: &mut MipsCpu<T>);
    fn attach(&mut self, cpu: &mut MipsCpu<T>);

    fn start(&mut self, cpu: &mut MipsCpu<T>) -> bool;
    fn stop(&mut self, cpu: &mut MipsCpu<T>);
    fn on_emu_panic(&mut self);
    fn on_syscall(&mut self, id: u32, cpu: &mut MipsCpu<T>) -> bool;
    fn on_break(&mut self, id: u32, cpu: &mut MipsCpu<T>) -> bool;
    fn check_memory_access(&mut self, cpu: &mut MipsCpu<T>) -> bool;
    fn check_syscall_access(&mut self, cpu: &mut MipsCpu<T>) -> bool;
    fn on_memory_read(&mut self, addr: u32, len: u32, cpu: &mut MipsCpu<T>) -> bool;
    fn on_memory_write(&mut self, addr: u32, len: u32, cpu: &mut MipsCpu<T>) -> bool;

    fn memory_error(&mut self, error_id: u32, cpu: &mut MipsCpu<T>);
    fn arithmitic_error(&mut self, error_id: u32, cpu: &mut MipsCpu<T>);
    fn invalid_op_code(&mut self, cpu: &mut MipsCpu<T>);
}

//-------------------------------------------------------- co processors
#[repr(align(4096))]
//#[repr(C)]
pub struct MipsCpu<T: CpuExternalHandler> {
    pc: u32,
    reg: [u32; 32],
    lo: u32,
    hi: u32,
    check: bool,
    running: bool,
    finished: bool,
    is_paused: bool,
    _cp0: CP0,
    _cp1: CP1,

    mem: SharedPagePoolMemory<Memory>,
    instructions_ran: u64,
    paused: AtomicUsize,
    _inturupts: Mutex<Vec<()>>,
    dropped: bool,
    external_handler: T,

    debugger: Arc<Mutex<Option<Box<dyn Debugger<T>>>>>,
}

impl<T: CpuExternalHandler> MipsCpu<T> {
    #[inline(always)]
    pub fn mem(&mut self) -> &mut SharedPagePoolMemory<Memory> {
        &mut self.mem
    }
    #[inline(always)]
    pub fn pc(&self) -> u32 {
        self.pc
    }
    #[inline(always)]
    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }
    #[inline(always)]
    pub fn reg(&self) -> &[u32; 32] {
        &self.reg
    }
    #[inline(always)]
    pub fn reg_mut(&mut self) -> &mut [u32; 32] {
        &mut self.reg
    }
    #[inline(always)]
    pub fn reg_num(&self, reg: usize) -> u32 {
        self.reg[reg]
    }
    #[inline(always)]
    pub fn lo(&self) -> u32 {
        self.lo
    }
    #[inline(always)]
    pub fn hi(&self) -> u32 {
        self.hi
    }
}

///
/// # Safety
///
/// `cpu` contains the actual value of `&mut self`, misusing `cpu` can cause unintended mutations of `self`
pub unsafe trait CpuExternalHandler: Sync + Send + Sized + 'static {
    fn arithmetic_error(&mut self, cpu: &mut MipsCpu<Self>, error_id: u32);
    fn memory_error(&mut self, cpu: &mut MipsCpu<Self>, error_id: u32);
    fn invalid_opcode(&mut self, cpu: &mut MipsCpu<Self>);
    fn system_call(&mut self, cpu: &mut MipsCpu<Self>, call_id: u32);
    fn breakpoint(&mut self, cpu: &mut MipsCpu<Self>, call_id: u32);
    fn system_call_error(
        &mut self,
        cpu: &mut MipsCpu<Self>,
        call_id: u32,
        error_id: u32,
        message: &str,
    );
    fn pause_block(cpu: &mut MipsCpu<Self>, fn_once: impl FnOnce(&MipsCpu<Self>)) {
        //this assumes that we are IN a system call and MipsCpu::run() isnt running somewhere else
        cpu.paused
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        cpu.is_paused = true;
        fn_once(cpu);
        cpu.paused
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        cpu.is_paused = false;
        while *cpu.paused.get_mut() > 0 {
            std::hint::spin_loop();
        }
    }
    fn cpu_stop(&mut self) {}
    fn cpu_start(&mut self) {}
    fn cpu_pause(&mut self) {}
    fn cpu_resume(&mut self) {}
}

#[derive(Default)]
pub struct DefaultExternalHandler {}

impl DefaultExternalHandler {
    fn opcode_address(cpu: &mut MipsCpu<Self>) -> u32 {
        cpu.pc.wrapping_sub(4)
    }

    fn opcode(cpu: &mut MipsCpu<Self>) -> u32 {
        unsafe { cpu.mem.get_u32_alligned_be(cpu.pc.wrapping_sub(4)) }
    }
}

unsafe impl CpuExternalHandler for DefaultExternalHandler {
    fn arithmetic_error(&mut self, cpu: &mut MipsCpu<Self>, error_id: u32) {
        log::warn!("arithmetic error {}", error_id);
        cpu.stop();
    }

    fn memory_error(&mut self, cpu: &mut MipsCpu<Self>, error_id: u32) {
        log::warn!("Memory Error: {}", error_id);
        cpu.stop();
    }

    fn invalid_opcode(&mut self, cpu: &mut MipsCpu<Self>) {
        log::warn!(
            "invalid opcode {:#08X} at {:#08X}",
            Self::opcode(cpu),
            Self::opcode_address(cpu)
        );
        cpu.stop();
    }

    fn system_call(&mut self, cpu: &mut MipsCpu<Self>, call_id: u32) {
        match call_id {
            0 => cpu.stop(),
            1 => log::info!("{}", cpu.reg[4] as i32),
            4 => {
                let _address = cpu.reg[4];
            }
            5 => {
                let mut string = String::new();
                let _ = std::io::stdin().read_line(&mut string);
                match string.parse::<i32>() {
                    Ok(val) => cpu.reg[2] = val as u32,
                    Err(_) => match string.parse::<u32>() {
                        Ok(val) => cpu.reg[2] = val,
                        Err(_) => {
                            cpu.system_call_error(call_id, 0, "unable to parse integer");
                        }
                    },
                }
            }
            99 => {}
            101 => match char::from_u32(cpu.reg[4]) {
                Some(val) => log::info!("{}", val),
                None => log::warn!("Invalid char{}", cpu.reg[4]),
            },
            102 => {
                let mut string = String::new();
                let _ = std::io::stdin().read_line(&mut string);
                string = string.replace('\n', "");
                string = string.replace('\r', "");
                if string.len() != 1 {
                    cpu.reg[2] = string.chars().next().unwrap() as u32;
                } else {
                    cpu.system_call_error(call_id, 0, "invalid input");
                }
            }
            105 => {
                use std::thread;
                thread::sleep(Duration::from_millis(cpu.reg[4] as u64));
            }
            106 => {
                static mut LAST: u128 = 0;
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let dur = unsafe { time - LAST };
                cpu.reg[4] *= 2;
                if (cpu.reg[4] as u128) >= dur {
                    std::thread::sleep(std::time::Duration::from_millis(
                        (cpu.reg[4] as u64) - (dur as u64),
                    ));
                    unsafe {
                        LAST = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                    }
                } else {
                    unsafe {
                        LAST = time;
                    }
                }
            }
            107 => {
                cpu.reg[2] = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
                    & 0xFFFFFFFFu128) as u32;
            }
            130 => {
                cpu.reg[2] = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros()
                    & 0xFFFFFFFFu128) as u32;
            }
            111 => {
                cpu.stop();
            }
            _ => {
                self.system_call_error(cpu, call_id, 0, "invalid system call");
            }
        }
    }

    fn system_call_error(
        &mut self,
        _cpu: &mut MipsCpu<Self>,
        call_id: u32,
        error_id: u32,
        message: &str,
    ) {
        log::warn!(
            "System Call: {} Error: {} Message: {}",
            call_id,
            error_id,
            message
        );
    }

    fn breakpoint(&mut self, cpu: &mut MipsCpu<Self>, _call_id: u32) {
        cpu.stop();
    }
}

// impl<T: CpuExternalHandler> PagePoolListener for MipsCpu<T> {
//     fn lock(&mut self, _initiator: bool) -> Result<(), Box<dyn std::error::Error>> {
//         self.pause_exclude_memory_event();
//         Result::Ok(())
//     }

//     fn unlock(&mut self, _initiator: bool) -> Result<(), Box<dyn std::error::Error>> {
//         self.resume();
//         Result::Ok(())
//     }
// }

impl<T: CpuExternalHandler> Drop for MipsCpu<T> {
    fn drop(&mut self) {
        self.dropped = true;
        self.stop_and_wait();
    }
}

impl<T: CpuExternalHandler> MipsCpu<T> {
    #[allow(unused)]
    pub fn new_interface(handler: T) -> EmulatorInterface<T> {
        let mut tmp = MipsCpu {
            instructions_ran: 0,
            pc: 0,
            reg: [0; 32],
            _cp0: CP0::default(),
            _cp1: CP1::default(),
            lo: 0,
            hi: 0,
            check: false,
            running: false,
            finished: true,
            paused: 0.into(),
            is_paused: true,
            // is_within_memory_event: false,
            mem: Memory::new(),
            external_handler: handler,
            _inturupts: Default::default(),
            dropped: false,
            debugger: Arc::new(Mutex::new(None)),
        };

        let mut interface = EmulatorInterface::new(tmp);

        let clone = interface.clone();
        interface.cpu_mut(|cpu| {
            cpu.mem.set_emulator_pause(clone);
        });

        interface
    }

    #[allow(unused)]
    pub fn get_mem<M: PagedMemoryImpl + Default + Send + Sync + 'static>(
        &mut self,
    ) -> SharedPagePoolMemory<M> {
        self.get_mem_controller()
            .lock()
            .unwrap()
            .add_holder(Box::new(M::default()))
    }

    /// # Safety
    ///
    /// using this while the emulator is running can cause race conditions everywhere and break the state of the cpu
    pub unsafe fn raw_handler(&mut self) -> &mut T {
        &mut self.external_handler
    }

    #[allow(unused)]
    pub fn get_mem_controller(&mut self) -> std::sync::Arc<std::sync::Mutex<PagePoolController>> {
        match &self.mem.page_pool {
            Some(val) => val.clone_page_pool_mutex(),
            None => panic!(),
        }
    }

    pub fn instructions_ran(&self) -> u64 {
        self.instructions_ran
    }

    #[allow(unused)]
    pub fn is_running(&self) -> bool {
        unsafe {
            *core::ptr::read_volatile(&&self.running) || !*core::ptr::read_volatile(&&self.finished)
        }
    }

    #[allow(unused)]
    pub fn is_going_to_stop(&self) -> bool {
        unsafe {
            !*core::ptr::read_volatile(&&self.running)
        }
    }

    pub fn paused_or_stopped(&self) -> bool {
        self.is_paused() || !self.is_running()
    }

    #[inline(always)]
    pub fn is_paused(&self) -> bool {
        unsafe {
            *core::ptr::read_volatile(&&self.is_paused)
                && self.paused.load(std::sync::atomic::Ordering::Relaxed) > 0
        }
    }

    pub fn is_being_dropped(&self) -> bool {
        unsafe { *core::ptr::read_volatile(&&self.dropped) }
    }

    // fn is_within_memory_event(&self) -> bool {
    //     unsafe { *core::ptr::read_volatile(&&self.is_within_memory_event) }
    // }

    #[allow(unused)]
    pub fn stop(&mut self) {
        unsafe {
            core::ptr::write_volatile(&mut self.running, false);
            core::ptr::write_volatile(&mut self.check, true);
        }
    }

    pub fn stop_and_wait(&mut self) {
        self.stop();
        while self.is_running() {
            std::hint::spin_loop();
        }
    }

    #[allow(unused)]
    pub fn reset(&mut self) {
        self.pc = 0;
        self.reg = [0; 32];
        self.lo = 0;
        self.hi = 0;
        self.instructions_ran = 0;
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.reset();
        self.mem.unload_all_pages();
    }

    fn pause(&mut self) {
        self.paused
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        while unsafe {
            core::ptr::write_volatile(&mut self.check, true);
            !self.is_paused() && self.is_running()
        } {
            std::hint::spin_loop();
        }
    }

    pub fn attach_debugger(&mut self, new_debugger: impl Debugger<T>) {
        self.pause();
        let c = self.debugger.clone();
        let mut debugger = c.lock().unwrap();

        let mut old_debugger = None;
        std::mem::swap(&mut old_debugger, &mut *debugger);
        if let Some(mut debugger) = old_debugger {
            debugger.detach(self);
        }

        *debugger = Some(Box::new(new_debugger));
        debugger.as_mut().unwrap().attach(self);
        drop(debugger);
        self.resume();
    }

    pub fn detach_debugger(&mut self) {
        self.pause();
        let c = self.debugger.clone();
        let mut debugger = c.lock().unwrap();

        let mut old_debugger = None;
        std::mem::swap(&mut old_debugger, &mut *debugger);
        if let Some(mut debugger) = old_debugger {
            debugger.detach(self);
        }
        drop(debugger);
        self.resume();
    }

    fn resume(&mut self) {
        if self.paused.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            self.paused
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    pub fn run_panic(&mut self) {
        self.running = false;
        self.finished = true;
        self.check = false;
        self.is_paused = true;
        //self.clear();
        
        let mut debugger = self.debugger.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(debugger) = &mut *debugger{
            debugger.on_emu_panic();
        }
        *debugger = debugger.take();
        self.debugger.clear_poison();
        drop(debugger);
    }

    #[inline(never)]
    #[cold]
    fn system_call_error(&mut self, call_id: u32, error_id: u32, message: &str) {
        unsafe { core::mem::transmute::<&mut T, &mut T>(&mut self.external_handler) }
            .system_call_error(self, call_id, error_id, message);
    }
    #[inline(never)]
    #[cold]
    fn memory_error(&mut self, error_id: u32) {
        self.if_has_debugger(|cpu, debugger| {
            debugger.memory_error(error_id, cpu);
        });
        unsafe { core::mem::transmute::<&mut T, &mut T>(&mut self.external_handler) }
            .memory_error(self, error_id);
    }

    #[inline(never)]
    #[cold]
    fn arithmetic_error(&mut self, error_id: u32) {
        self.if_has_debugger(|cpu, debugger| {
            debugger.arithmitic_error(error_id, cpu);
        });
        unsafe { core::mem::transmute::<&mut T, &mut T>(&mut self.external_handler) }
            .arithmetic_error(self, error_id);
    }

    #[inline(never)]
    #[cold]
    fn invalid_op_code(&mut self) {
        self.if_has_debugger(|cpu, debugger| {
            debugger.invalid_op_code(cpu);
        });
        unsafe { core::mem::transmute::<&mut T, &mut T>(&mut self.external_handler) }
            .invalid_opcode(self);
    }

    fn system_call(&mut self, call_id: u32) {
        unsafe { core::mem::transmute::<&mut T, &mut T>(&mut self.external_handler) }
            .system_call(self, call_id);
    }

    fn breakpoint(&mut self, call_id: u32) {
        unsafe { core::mem::transmute::<&mut T, &mut T>(&mut self.external_handler) }
            .breakpoint(self, call_id);
    }

    fn if_has_debugger<R>(
        &mut self,
        fn_once: impl FnOnce(&mut Self, &mut Box<dyn Debugger<T>>) -> R,
    ) {
        let c = self.debugger.clone();
        let lock = c.lock();
        if let Ok(mut guard) = lock {
            if let Some(debugger) = &mut *guard {
                fn_once(self, debugger);
            }
        }
    }

    #[allow(dead_code)]
    pub fn start_new_thread(&'static mut self) {
        if self.running || !self.finished {
            return;
        }

        //stack overflows occur so we need to make a custom thread with a larger stack to account for that and it fixes the problem
        let _handle = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(move || {
                // some work here
                log::info!("CPU Started");
                let start = std::time::SystemTime::now();

                let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    self.run(false);
                }));

                let since_the_epoch = std::time::SystemTime::now()
                    .duration_since(start)
                    .expect("Time went backwards");
                log::info!("CPU stopping");
                log::debug!("time ran for: {:?}", since_the_epoch);

                match result {
                    Ok(_) => {}
                    Err(_) => {
                        self.run_panic();
                    }
                }
            });
    }

    #[allow(dead_code)]
    pub fn step_new_thread(&'static mut self) {
        if self.running || !self.finished {
            return;
        }

        let _handle = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                log::info!("CPU Step Started");
                let start = std::time::SystemTime::now();

                let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    self.run(true);
                }));

                let since_the_epoch = std::time::SystemTime::now()
                    .duration_since(start)
                    .expect("Time went backwards");
                log::info!("{:?}", since_the_epoch);
                log::info!("CPU Step Stopping");

                match result {
                    Ok(_) => {}
                    Err(_err) => {
                        self.run_panic();
                    }
                }
            });
    }

    #[allow(dead_code)]
    pub fn start_local(&mut self) {
        if self.running || !self.finished {
            return;
        }

        log::info!("CPU Started");
        self.run(false);
        log::info!("CPU Step Stopping");
    }

    #[allow(dead_code)]
    pub fn step_local(&'static mut self) {
        if self.running || !self.finished {
            return;
        }

        log::info!("CPU Step Started");

        self.run(true);

        log::info!("CPU Step Stopped");
    }
}

//------------------------------------------------------------------------------------------------------------------------

macro_rules! core_emu {
    ($self:ident, $debugger_lock:ident, $id:ident, $address:ident, $bp:block, $sc:block, $rb:block, $rhw:block, $rw:block, $wb:block, $whw:block, $ww:block) => {
        let mut ins_cache = unsafe {
            (
                &mut ($self.mem.get_or_make_page($self.pc).as_mut()).page,
                $self.pc >> 16,
            )
        };
        let mut mem_cache =
            unsafe { (&mut ($self.mem.get_or_make_page(0).as_mut()).page, 0u32) };

        macro_rules! set_mem_alligned {
            ($add:expr, $val:expr, $fn_type:ty) => {
                unsafe {
                    let address = $add;
                    if core::intrinsics::unlikely(address >> 16 != mem_cache.1) {
                        mem_cache = (
                            &mut ($self.mem.get_or_make_page(address).as_mut()).page,
                            address >> 16,
                        );
                    }

                    let item = mem_cache.0.get_unchecked_mut(address as u16 as usize);
                    *core::mem::transmute::<&mut u8, &mut $fn_type>(item) = $val.to_be()
                }
            };
        }

        macro_rules! get_mem_alligned {
            ($add:expr, $fn_type:ty) => {
                unsafe {
                    let address = $add;
                    if core::intrinsics::unlikely(address >> 16 != mem_cache.1) {
                        mem_cache = (
                            &mut ($self.mem.get_or_make_page(address).as_mut()).page,
                            address >> 16,
                        );
                    }

                    let item = mem_cache.0.get_unchecked(address as u16 as usize);
                    core::mem::transmute::<&u8, &$fn_type>(item).to_be()
                }
            };
        }


        'cpu_loop: while {
            let op: u32 = unsafe {
                if core::intrinsics::unlikely($self.pc >> 16 != ins_cache.1) {
                    ins_cache = (
                        &mut ($self.mem.get_or_make_page($self.pc).as_mut()).page,
                        $self.pc >> 16,
                    );
                }

                let item = ins_cache.0.get_unchecked($self.pc as u16 as usize);
                core::mem::transmute::<&u8, &u32>(item).to_be()
            };

            //prevent overflow
            $self.pc = $self.pc.wrapping_add(4);
            $self.instructions_ran += 1;

            {
                macro_rules! get_reg {
                    ($reg:expr) => {
                        unsafe { *$self.reg.get_unchecked($reg) }
                    };
                }

                match op >> 26 {
                    0 => {
                        match op & 0b111111 {
                            // REGISTER formatted instructions

                            //arithmatic
                            0b100000 => {
                                //ADD
                                match ($self.reg[register_s!(op)] as i32)
                                    .checked_add($self.reg[register_t!((op))] as i32)
                                {
                                    Some(val) => {
                                        $self.reg[register_d!(op)] = val as u32;
                                    }

                                    None => {
                                        drop($debugger_lock);
                                        $self.arithmetic_error(2);
                                        break 'cpu_loop;
                                    }
                                }
                            }
                            0b100001 => {
                                //ADDU
                                $self.reg[register_d!(op)] = $self.reg[register_s!(op)]
                                    .wrapping_add($self.reg[register_t!((op))])
                            }
                            0b100100 => {
                                //AND
                                $self.reg[register_d!(op)] =
                                    $self.reg[register_s!(op)] & $self.reg[register_t!((op))]
                            }
                            0b011010 => {
                                //DIV
                                let t = $self.reg[register_t!(op)] as i32;
                                if core::intrinsics::likely(t != 0) {
                                    let s = $self.reg[register_s!(op)] as i32;
                                    $self.lo = (s.wrapping_div(t)) as u32;
                                    $self.hi = (s.wrapping_rem(t)) as u32;
                                } else {
                                    drop($debugger_lock);
                                    $self.arithmetic_error(0);
                                    break 'cpu_loop;
                                }
                            }
                            0b011011 => {
                                //DIVU
                                let t = $self.reg[register_t!(op)];
                                if core::intrinsics::likely(t != 0) {
                                    let s = $self.reg[register_s!(op)];
                                    $self.lo = s.wrapping_div(t);
                                    $self.hi = s.wrapping_rem(t);
                                } else {
                                    drop($debugger_lock);
                                    $self.arithmetic_error(0);
                                    break 'cpu_loop;
                                }
                            }
                            0b011000 => {
                                //MULT
                                let t = $self.reg[register_t!(op)] as i32 as i64;
                                let s = $self.reg[register_s!(op)] as i32 as i64;
                                let result = t.wrapping_mul(s);
                                $self.lo = (result & 0xFFFFFFFF) as u32;
                                $self.hi = (result >> 32) as u32;
                            }
                            0b011001 => {
                                //MULTU
                                let t = $self.reg[register_t!(op)] as u64;
                                let s = $self.reg[register_s!(op)] as u64;
                                let result = t.wrapping_mul(s);
                                $self.lo = (result & 0xFFFFFFFF) as u32;
                                $self.hi = (result >> 32) as u32;
                            }
                            0b100111 => {
                                //NOR
                                $self.reg[register_d!(op)] =
                                    !($self.reg[register_s!(op)] | $self.reg[register_t!(op)]);
                            }
                            0b100101 => {
                                //OR
                                $self.reg[register_d!(op)] =
                                    $self.reg[register_s!(op)] | $self.reg[register_t!(op)];
                            }
                            0b100110 => {
                                //XOR
                                $self.reg[register_d!(op)] =
                                    $self.reg[register_s!(op)] ^ $self.reg[register_t!(op)];
                            }
                            0b000000 => {
                                //SLL
                                $self.reg[register_d!(op)] =
                                    $self.reg[register_t!(op)] << register_a!(op);
                            }
                            0b000100 => {
                                //SLLV
                                $self.reg[register_d!(op)] = ($self.reg[register_t!(op)])
                                    << (0b11111 & $self.reg[register_s!(op)]);
                            }
                            0b000011 => {
                                //SRA
                                $self.reg[register_d!(op)] = ($self.reg[register_t!(op)] as i32
                                    >> register_a!(op))
                                    as u32;
                            }
                            0b000111 => {
                                //SRAV
                                $self.reg[register_d!(op)] = ($self.reg[register_t!(op)] as i32
                                    >> (0b11111 & $self.reg[register_s!(op)]))
                                    as u32;
                            }
                            0b000010 => {
                                //SRL
                                $self.reg[register_d!(op)] =
                                    ($self.reg[register_t!(op)] >> register_a!(op)) as u32;
                            }
                            0b000110 => {
                                //SRLV
                                $self.reg[register_d!(op)] = ($self.reg[register_t!(op)]
                                    >> (0b11111 & $self.reg[register_s!(op)]))
                                    as u32;
                            }
                            0b100010 => {
                                //SUB
                                if let Option::Some(val) = ($self.reg[register_s!(op)] as i32)
                                    .checked_sub($self.reg[register_t!(op)] as i32)
                                {
                                    $self.reg[register_d!(op)] = val as u32;
                                } else {
                                    drop($debugger_lock);
                                    $self.arithmetic_error(1);
                                    break 'cpu_loop;
                                }
                            }
                            0b100011 => {
                                //SUBU
                                $self.reg[register_d!(op)] = $self.reg[register_s!(op)]
                                    .wrapping_sub($self.reg[register_t!(op)]);
                            }

                            //comparason
                            0b101010 => {
                                //SLT
                                $self.reg[register_d!(op)] = {
                                    if ($self.reg[register_s!(op)] as i32)
                                        < ($self.reg[register_t!(op)] as i32)
                                    {
                                        1
                                    } else {
                                        0
                                    }
                                }
                            }
                            0b101011 => {
                                //SLTU
                                $self.reg[register_d!(op)] = {
                                    if $self.reg[register_s!(op)] < $self.reg[register_t!(op)] {
                                        1
                                    } else {
                                        0
                                    }
                                }
                            }

                            //jump
                            0b001001 => {
                                //JALR
                                $self.reg[31] = $self.pc;
                                $self.pc = $self.reg[register_s!(op)];
                            }
                            0b001000 => {
                                //JR
                                $self.pc = $self.reg[register_s!(op)];
                            }

                            //data movement
                            0b010000 => {
                                //MFHI
                                $self.reg[register_d!(op)] = $self.hi;
                            }
                            0b010010 => {
                                //MFLO
                                $self.reg[register_d!(op)] = $self.lo;
                            }
                            0b010001 => {
                                //MTHI
                                $self.hi = $self.reg[register_s!(op)];
                            }
                            0b010011 => {
                                //MTLO
                                $self.lo = $self.reg[register_s!(op)];
                            }

                            //special
                            0b001100 => {
                                //syscall
                                let $id = (op >> 6) & 0b11111111111111111111;
                                $sc
                                $self.system_call($id);
                            }
                            0b001101 => {
                                //break
                                let $id = (op >> 6) & 0b11111111111111111111;
                                $bp
                                $self.breakpoint($id);
                            }
                            0b110100 => {
                                //TEQ
                                if $self.reg[register_s!(op)] == $self.reg[register_t!(op)] {
                                    let $id = (op >> 6) & 0b1111111111;
                                    $sc
                                    $self.system_call($id);
                                }
                            }
                            0b110000 => {
                                //TGE
                                if $self.reg[register_s!(op)] as i32
                                    >= $self.reg[register_t!(op)] as i32
                                {
                                    let $id = (op >> 6) & 0b1111111111;
                                    $sc
                                    $self.system_call($id)
                                }
                            }
                            0b110001 => {
                                //TGEU
                                if $self.reg[register_s!(op)] >= $self.reg[register_t!(op)] {
                                    let $id = (op >> 6) & 0b1111111111;
                                    $sc
                                    $self.system_call($id)
                                }
                            }
                            0b110010 => {
                                //TIT
                                if ($self.reg[register_s!(op)] as i32)
                                    < $self.reg[register_t!(op)] as i32
                                {
                                    let $id = (op >> 6) & 0b1111111111;
                                    $sc
                                    $self.system_call($id)
                                }
                            }
                            0b110011 => {
                                //TITU
                                if $self.reg[register_s!(op)] < $self.reg[register_t!(op)] {
                                    let $id = (op >> 6) & 0b1111111111;
                                    $sc
                                    $self.system_call($id)
                                }
                            }
                            0b110110 => {
                                //TNE
                                if $self.reg[register_s!(op)] != $self.reg[register_t!(op)] {
                                    let $id = (op >> 6) & 0b1111111111;
                                    $sc
                                    $self.system_call($id)
                                }
                            }

                            _ => {
                                drop($debugger_lock);
                                $self.invalid_op_code();
                                break 'cpu_loop;
                            },
                        }
                    }
                    //Jump instructions
                    0b000010 => {
                        //jump
                        $self.pc = ($self.pc & 0b11110000000000000000000000000000)
                            | jump_immediate_address!(op);
                    }
                    0b000011 => {
                        //jal
                        $self.reg[31] = $self.pc;
                        $self.pc = ($self.pc & 0b11110000000000000000000000000000)
                            | jump_immediate_address!(op);
                    }
                    // IMMEDIATE formmated instructions

                    // arthmetic
                    0b001000 => {
                        //ADDI
                        if let Option::Some(val) = ($self.reg[immediate_s!(op)] as i32)
                            .checked_add(immediate_immediate_signed_extended!(op) as i32)
                        {
                            $self.reg[immediate_t!(op)] = val as u32;
                        } else {
                            drop($debugger_lock);
                            $self.arithmetic_error(1);
                            break 'cpu_loop;
                        }
                    }
                    0b001001 => {
                        //ADDIU
                        $self.reg[immediate_t!(op)] = ($self.reg[immediate_s!(op)])
                            .wrapping_add(immediate_immediate_signed_extended!(op));
                    }
                    0b001100 => {
                        //ANDI
                        $self.reg[immediate_t!(op)] =
                            $self.reg[immediate_s!(op)] & immediate_immediate_zero_extended!(op)
                    }
                    0b001101 => {
                        //ORI
                        $self.reg[immediate_t!(op)] = $self.reg[immediate_s!(op)] as u32
                            | immediate_immediate_zero_extended!(op) as u32
                    }
                    0b001110 => {
                        //XORI
                        $self.reg[immediate_t!(op)] = $self.reg[immediate_s!(op)] as u32
                            ^ immediate_immediate_zero_extended!(op) as u32
                    }

                    // constant manupulating inctructions
                    0b001111 => {
                        //LUI
                        $self.reg[immediate_t!(op)] =
                            immediate_immediate_zero_extended!(op) << 16;
                    }
                    // these were replaced
                    // 0b011001 => {
                    //     //LHI
                    //     let t = immediate_t!(op) as usize;
                    //     $self.reg[t] =
                    //         $self.reg[t] & 0xFFFF | immediate_immediate_unsigned_hi!(op) as u32;
                    // }
                    // 0b011000 => {
                    //     //LLO
                    //     let t = immediate_t!(op) as usize;
                    //     $self.reg[t] =
                    //         $self.reg[t] & 0xFFFF0000 | immediate_immediate_zero_extended!(op) as u32;
                    // }

                    // comparison Instructions
                    0b001010 => {
                        //SLTI
                        $self.reg[immediate_t!(op)] = {
                            if ($self.reg[immediate_s!(op)] as i32)
                                < (immediate_immediate_signed_extended!(op) as i32)
                            {
                                1
                            } else {
                                0
                            }
                        }
                    }
                    0b001011 => {
                        //SLTIU
                        $self.reg[immediate_t!(op)] = {
                            if ($self.reg[immediate_s!(op)] as u32)
                                < (immediate_immediate_signed_extended!(op) as u32)
                            {
                                1
                            } else {
                                0
                            }
                        }
                    }

                    // branch instructions
                    0b000100 => {
                        //BEQ
                        if get_reg!(immediate_s!(op)) == get_reg!(immediate_t!(op)) {
                            $self.pc = (($self.pc as i32)
                                .wrapping_add(immediate_immediate_address!(op)))
                                as u32;
                        }
                    }
                    0b000001 => {
                        match immediate_t!(op) {
                            0b00001 => {
                                //BGEZ
                                if ($self.reg[immediate_s!(op)] as i32) >= 0 {
                                    $self.pc = (($self.pc as i32)
                                        .wrapping_add(immediate_immediate_address!(op)))
                                        as u32;
                                }
                            }
                            0b00000 => {
                                //BLTZ
                                if ($self.reg[immediate_s!(op)] as i32) < 0 {
                                    $self.pc = (($self.pc as i32)
                                        .wrapping_add(immediate_immediate_address!(op)))
                                        as u32;
                                }
                            }
                            _ => {
                                $self.invalid_op_code();
                            }
                        }
                    }
                    0b000111 => {
                        //BGTZ
                        if $self.reg[immediate_s!(op)] as i32 > 0 {
                            $self.pc = (($self.pc as i32)
                                .wrapping_add(immediate_immediate_address!(op)))
                                as u32;
                        }
                    }

                    0b000110 => {
                        //BLEZ
                        if $self.reg[immediate_s!(op)] as i32 <= 0 {
                            $self.pc = (($self.pc as i32)
                                .wrapping_add(immediate_immediate_address!(op)))
                                as u32;
                        }
                    }
                    0b000101 => {
                        //BNE
                        if $self.reg[immediate_s!(op)] != $self.reg[immediate_t!(op) as usize] {
                            $self.pc = (($self.pc as i32)
                                .wrapping_add(immediate_immediate_address!(op)))
                                as u32;
                        }
                    }

                    //load unsinged instructions
                    0b100010 => {
                        //LWL
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        let reg_num = immediate_t!(op);
                        let thing: &mut [u8; 4] =
                            unsafe { core::mem::transmute(&mut $self.reg[reg_num]) };
                        $rhw
                        thing[3] = get_mem_alligned!($address, u8); //$self.mem.get_u8(address);
                        thing[2] = get_mem_alligned!($address.wrapping_add(1), u8);
                        //$self.mem.get_u8(address + 1);
                    }
                    0b100110 => {
                        //LWR
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        let reg_num = immediate_t!(op);
                        let thing: &mut [u8; 4] =
                            unsafe { core::mem::transmute(&mut $self.reg[reg_num]) };
                        {
                            let $address = $address.wrapping_sub(1);
                            $rhw
                        }
                        thing[0] = get_mem_alligned!($address, u8); //$self.mem.get_u8(address);
                        thing[1] = get_mem_alligned!($address.wrapping_sub(1), u8);
                        //$self.mem.get_u8(address.wrapping_sub(1));
                    }

                    //save unaliged instructions
                    0b101010 => {
                        //SWL
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        let reg_num = immediate_t!(op);
                        let thing: [u8; 4] = $self.reg[reg_num].to_ne_bytes();
                        $whw
                        set_mem_alligned!($address, thing[3], u8);
                        set_mem_alligned!($address.wrapping_add(1), thing[2], u8);
                    }
                    0b101110 => {
                        //SWR
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        let reg_num = immediate_t!(op);
                        let thing: [u8; 4] = $self.reg[reg_num].to_ne_bytes();
                        {
                            let $address = $address.wrapping_sub(1);
                            $whw
                        }
                        set_mem_alligned!($address, thing[0], u8);
                        set_mem_alligned!($address.wrapping_sub(1), thing[1], u8);
                    }

                    // load instrictions
                    0b100000 => {
                        //LB
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        $rb
                        $self.reg[immediate_t!(op)] = get_mem_alligned!($address, i8) as u32;
                    }
                    0b100100 => {
                        //LBU
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        $rb
                        $self.reg[immediate_t!(op)] = get_mem_alligned!($address, u8) as u32;
                    }
                    0b100001 => {
                        //LH
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;

                        if core::intrinsics::likely($address & 0b1 == 0) {
                            $rhw
                            $self.reg[immediate_t!(op)] = get_mem_alligned!($address, i16) as u32;
                        //$self.mem.get_i16_alligned(address) as u32
                        } else {
                            drop($debugger_lock);
                            $self.memory_error(0);
                            break 'cpu_loop;
                        }
                    }
                    0b100101 => {
                        //LHU
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;

                        if core::intrinsics::likely($address & 0b1 == 0) {
                            $rhw
                            $self.reg[immediate_t!(op)] = get_mem_alligned!($address, u16) as u32;
                        //$self.mem.get_u16_alligned(address) as u32
                        } else {
                            drop($debugger_lock);
                            $self.memory_error(0);
                            break 'cpu_loop;
                        }
                    }
                    0b100011 => {
                        //LW
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;

                        if core::intrinsics::likely($address & 0b11 == 0) {
                            $rhw
                            $self.reg[immediate_t!(op)] = get_mem_alligned!($address, u32);
                        //$self.mem.get_u32_alligned(address) as u32
                        } else {
                            drop($debugger_lock);
                            $self.memory_error(1);
                            break 'cpu_loop;
                        }
                    }

                    // store instructions
                    0b101000 => {
                        //SB
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        $wb
                        set_mem_alligned!($address, $self.reg[immediate_t!(op)] as u8, u8);
                    }
                    0b101001 => {
                        //SH
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;

                        if core::intrinsics::likely($address & 0b1 == 0) {
                            $whw
                            set_mem_alligned!($address, $self.reg[immediate_t!(op)] as u16, u16);
                        } else {
                            drop($debugger_lock);
                            $self.memory_error(3);
                            break 'cpu_loop;
                        }
                    }
                    0b101011 => {
                        //SW
                        let $address = (($self.reg[immediate_s!(op)] as i32)
                            .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                            as u32;
                        if core::intrinsics::likely($address & 0b11 == 0) {
                            $ww
                            set_mem_alligned!($address, $self.reg[immediate_t!(op)], u32);
                        } else {
                            drop($debugger_lock);
                            $self.memory_error(4);
                            break 'cpu_loop;
                        }
                    }

                    _ => {
                        drop($debugger_lock);
                        $self.invalid_op_code();
                        break 'cpu_loop;
                    },
                }
            }

            !$self.check
        } {}
    };
}
//------------------------------------------------------------------------------------------------------------------------
impl<T: CpuExternalHandler> MipsCpu<T> {
    #[inline(never)]
    #[link_section = ".text.emu_run"]
    fn run(&mut self, step: bool) {
        let debugger = self.debugger.clone();
        let mut debugger = debugger.lock().unwrap();
        
        if let Some(debugger) = &mut *debugger {
            if debugger.start(self) {
                self.running = false;
                self.finished = true;
                return;
            }
        }
        if step{
            self.finished = false;
            self.running = false;
            self.check = true;
        }else{
            self.finished = false;
            self.running = true;
        }
        drop(debugger);
        
        self.external_handler.cpu_start();

        self.is_paused = false;
        'run_loop: while {
            while *self.paused.get_mut() > 0 {
                if !self.is_paused {
                    self.is_paused = true;
                    self.external_handler.cpu_pause();
                }
                if !self.running {
                    break 'run_loop;
                }
                std::hint::spin_loop();
            }
            if self.is_paused {
                self.is_paused = false;
                self.external_handler.cpu_resume();
            }

            let d = self.debugger.clone();
            let debugger_lock = d.lock().unwrap();

            if debugger_lock.is_none() {
                #[allow(unused)]
                {
                    core_emu!(
                        self,
                        debugger_lock,
                        id,
                        address,
                        {},
                        {},
                        {},
                        {},
                        {},
                        {},
                        {},
                        {}
                    );
                }
            } else if debugger_lock.is_some() {
                self.run_with_debugger(debugger_lock);
                impl<T: CpuExternalHandler> MipsCpu<T> {
                    #[inline(never)]
                    fn run_with_debugger(
                        &mut self,
                        mut debugger_lock: MutexGuard<'_, Option<Box<dyn Debugger<T>>>>,
                    ) {
                        let debugger = debugger_lock.as_mut().unwrap();
                        match (
                            debugger.check_memory_access(self),
                            debugger.check_syscall_access(self),
                        ) {
                            (true, true) => {
                                core_emu!(
                                    self,
                                    debugger_lock,
                                    id,
                                    address,
                                    {
                                        if debugger.on_break(id, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_syscall(id, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_read(address, 1, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_read(address, 2, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_read(address, 4, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_write(address, 1, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_write(address, 2, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_write(address, 4, self) {
                                            return; //break 'main_loop;
                                        }
                                    }
                                );
                            }
                            (true, false) => {
                                core_emu!(
                                    self,
                                    debugger_lock,
                                    id,
                                    address,
                                    {
                                        if debugger.on_break(id, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {},
                                    {
                                        if debugger.on_memory_read(address, 1, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_read(address, 2, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_read(address, 4, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_write(address, 1, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_write(address, 2, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_memory_write(address, 4, self) {
                                            return; //break 'main_loop;
                                        }
                                    }
                                );
                            }
                            #[allow(unused)]
                            (false, true) => {
                                core_emu!(
                                    self,
                                    debugger_lock,
                                    id,
                                    address,
                                    {
                                        if debugger.on_break(id, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {
                                        if debugger.on_syscall(id, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {},
                                    {},
                                    {},
                                    {},
                                    {},
                                    {}
                                );
                            }
                            #[allow(unused)]
                            (false, false) => {
                                core_emu!(
                                    self,
                                    debugger_lock,
                                    id,
                                    address,
                                    {
                                        if debugger.on_break(id, self) {
                                            return; //break 'main_loop;
                                        }
                                    },
                                    {},
                                    {},
                                    {},
                                    {},
                                    {},
                                    {},
                                    {}
                                );
                            } //_ => {}
                        }
                    }
                }
            }
            self.check = false;

            self.running
        } {}
        self.finished = true;

        self.external_handler.cpu_stop();

        let debugger = self.debugger.clone();
        let lock = debugger.lock();
        if let Ok(mut debugger) = lock {
            if let Some(debugger) = &mut *debugger {
                debugger.stop(self);
            }
        }
    }
}
