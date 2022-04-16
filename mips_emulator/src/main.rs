use std::{io::Read};

use mips_emulator::cpu::MipsCpu;


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
                        println!("CPU starting");
                        CPU_TEST.as_mut().unwrap().start_local();
                    }
                }
            }
            b'r' => {
                unsafe{
                    if CPU_TEST.as_mut().unwrap().is_running() {
                        println!("Cannot reset CPU while running");
                    }else{
                         //runs 2^16 * (2^15-1)*3+2 instructions (6442254338)
                        //the version written in c++ seems to be around 17% faster
                        //[0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000][(self.pc >> 2) as usize];//
                        
                        CPU_TEST.as_mut().unwrap().clear();

                        let test_prog = [
                            0x64020001u32,
                            0x00000820,
                            0x20210001,
                            0x10220001,
                            0x0BFFFFF0,
                            0x68000000,
                        ]; //
                        CPU_TEST
                            .as_mut()
                            .unwrap()
                            .get_mem()
                            .copy_into_raw(0, &test_prog);

                        //let test_prog = [0x64027FFFu32, 0x00000820, 0x20210001, 0x10220001, 0x0BFFFFFD, 0x68000000];//
                        CPU_TEST.as_mut().unwrap().get_mem().copy_into_raw(0, &test_prog);
                        
                        println!("reset CPU");
                    
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