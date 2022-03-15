use std::{thread, time::SystemTime, io::Read};

use cpu::MipsCpu;
mod memory;
mod cpu;

static mut CPU_TEST: Option<MipsCpu> = Option::None;

fn main(){

    unsafe{
        CPU_TEST = Option::Some(MipsCpu::new());
    }

    println!("s to start CPU, r to reset CPU, h to halt CPU and e to exit");
    'main_loop:
    loop{
        let mut b:[u8;1] = [0];
        let _result = std::io::stdin().read( b.as_mut_slice());
        match b[0]{
            b's' => {
                unsafe{
                    if CPU_TEST.as_mut().unwrap().is_running(){
                        println!("CPU is already running");
                    }else{
                        let _handle = thread::spawn(|| {
                            // some work here
                            println!("CPU Started");
                            let start = SystemTime::now();
                            CPU_TEST.as_mut().unwrap().start();
                            let since_the_epoch = SystemTime::now()
                                .duration_since(start)
                                .expect("Time went backwards");
                            println!("{:?}", since_the_epoch);
                            println!("CPU stopping");
                        });
                    }
                }
            }
            b'r' => {
                unsafe{
                    if CPU_TEST.as_mut().unwrap().is_running() {
                        println!("Cannot reset CPU while running");
                    }else{
                        println!("Reset CPU");
                        CPU_TEST.as_mut().unwrap().clear();
                    }
                }
            }
            b'h' => {
                unsafe{
                    if CPU_TEST.as_mut().unwrap().is_running() {
                        println!("Stopping CPU");
                        CPU_TEST.as_mut().unwrap().stop();
                    }else{
                        println!("CPU is not running");
                    }
                }
            }
            b'e' => {
                println!("Exiting");
                break 'main_loop;
            }
            b'\n' | b'\r' => {

            }
            _ => {
                println!("Invalid input");
            }
        }
    }
}
