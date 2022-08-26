use std::io::Read;

use mips_emulator::{
    cpu::{DefaultExternalHandler, MipsCpu},
    memory::{page_pool::MemoryDefault, single_cached_memory::SingleCachedMemory},
};

fn main() {
    // let test = &();
    // let test = test as *mut ();
    // let val = test;

    let mut emulator = MipsCpu::new(DefaultExternalHandler::default());

    println!("s to start CPU, r to reset CPU, h to halt CPU and e to exit");
    'main_loop: loop {
        let mut b: [u8; 1] = [0];
        let _result = std::io::stdin().read(b.as_mut_slice());
        match b[0] {
            b's' => {
                if emulator.start_new_thread().is_err() {
                    println!("CPU is already running");
                } else {
                    println!("CPU started");
                }
            }
            b'r' => {
                if emulator.restart().is_err() {
                    println!("Cannot reset CPU while running");
                } else {
                    emulator.cpu_mut(|cpu| {
                        let mut test_prog = [
                            0x3C027FFFu32,
                            0x00000820,
                            /*0x0AC01001C,*/ 0x24210001,
                            0x10220001,
                            0x08000002,
                            0x0000000C,
                        ];
                        for item in test_prog.iter_mut() {
                            *item = item.to_be();
                        }
                        //
                        unsafe {
                            cpu.get_mem::<SingleCachedMemory>()
                                .copy_into_raw(0, &test_prog);
                        }
                    });
                    println!("reset CPU");
                }
            }
            b'h' => {
                if emulator.stop().is_err() {
                    println!("CPU is not running");
                } else {
                    println!("Stopped CPU");
                }
            }
            b'p' => unsafe {
                let cpu = emulator.raw_cpu();
                println!("PC: {}", (*cpu).pc());
                println!("High: {}, Low: {}", (*cpu).hi(), (*cpu).lo());
                for i in 0..32 {
                    println!("${:02} = {}", i, (*cpu).reg()[i])
                }
            },
            b'e' => {
                println!("Exiting");
                break 'main_loop;
            }
            b'\n' | b'\r' => {}
            _ => {
                println!("Invalid input");
            }
        }
    }
}
