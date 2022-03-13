#[macro_use] extern crate lalrpop_util;
mod parsing_lexer;
mod virtual_cpu;
mod assembler;

use std::io::Read;
use std::thread;
use std::time::SystemTime;
//use parsing_lexer::ast::PrintVisitor;
//use parsing_lexer::gen_parser;
//use parsing_lexer::lexer::Lexer;
//use parsing_lexer::tokenizer::Tokenizer;
use virtual_cpu::cpu::MipsCpu;


static mut CPU_TEST: Option<MipsCpu> = Option::None;

fn do_it(){

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


fn test_tokenizer() {
    use parsing_lexer::tokenizer::Tokenizer;
    let file = std::fs::read_to_string("res/tests/tokenizer_test.cl").expect("bruh");

    let mut tokenizer = Tokenizer::from_string(&file).include_whitespace(true);
    let t = tokenizer.tokenize();
    for t in t{
        println!("token: {:?} string: {:?}", t, tokenizer.str_from_token(&t));
    }
}

fn main() {
    test_tokenizer();

    do_it();
    /*
    let file = std::fs::read_to_string("test2.cl").expect("bruh");
    println!("\nPrinting tokenizer_test.cl");

    let mut tokenizer = Tokenizer::new(&file);
    let tmp = tokenizer.tokenize();

    for t in tmp{
        println!("token: {} string: '{}'", t, tokenizer.str_from_token(&t));
    }
    tokenizer.reset();

    let mut parser = Parser::new(tokenizer);

    let result = parser.parse();

    match result {
        Ok(ok) => {
            let mut tests = PrintVisitor::new();
            ok.accept(Box::new(&mut tests));
            println!("{}", ok);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
     */
}


