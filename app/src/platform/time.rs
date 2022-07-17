use std::time::Duration;

pub fn duration_since_epoch() -> Duration {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let start = std::time::SystemTime::now();
        start
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
    }
    #[cfg(target_arch = "wasm32")]
    {
        let time: std::time::SystemTime;
        time = unsafe {  
            std::mem::transmute(Duration::from_millis(date_now() as u64))
        };
        time.duration_since(std::time::UNIX_EPOCH).expect("time went backwards")
    }
}


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}
