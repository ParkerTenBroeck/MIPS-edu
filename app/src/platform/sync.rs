use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::PoisonError;
pub trait PlatSpecificLocking{
    type Target;
    fn plat_lock(&self) -> Result<MutexGuard<'_, Self::Target>, PoisonError<MutexGuard<'_, Self::Target>>>;
}

impl<T> PlatSpecificLocking for Mutex<T>{
    type Target = T;
    #[cfg(not(target_arch = "wasm32"))]
    fn plat_lock(&self) -> Result<MutexGuard<'_, Self::Target>, PoisonError<MutexGuard<'_, Self::Target>>> {
        self.lock()
        
    }
    #[cfg(target_arch = "wasm32")]
    fn plat_lock(&self) -> Result<MutexGuard<'_, Self::Target>, PoisonError<MutexGuard<'_, Self::Target>>> {
        
        loop{
            match self.try_lock(){
                Ok(val) => {
                    return Ok(val);
                },
                Err(val) => {
                    match val{
                        std::sync::TryLockError::Poisoned(val) => {
                            return Err(val);
                        },
                        std::sync::TryLockError::WouldBlock => {
                            std::hint::spin_loop();
                        },
                    }
                },
            }
        }
    }
}