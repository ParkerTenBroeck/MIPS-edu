#[macro_use] extern crate lalrpop_util;
mod parsing_lexer;
mod virtual_cpu;

use std::thread;
use std::time::SystemTime;
use parsing_lexer::ast::*;
use parsing_lexer::gen_parser;
use parsing_lexer::lexer::Lexer;
use parsing_lexer::tokenizer::Tokenizer;
use virtual_cpu::cpu::MipsCpu;


static mut CPU_TEST: Option<MipsCpu> = Option::None;



fn do_it(){
    #[allow(unused_must_use)]
    unsafe {
        CPU_TEST = Option::Some(MipsCpu::new());
        //for i in 0..65536{
        //    CPU_TEST.as_mut().unwrap().mem.get_u32_alligned(i << 16);
        //}

        //let handle = thread::spawn(|| {
            // some work here
            println!("start thread");
            let start = SystemTime::now();
            CPU_TEST.as_mut().unwrap().start();
            let since_the_epoch = SystemTime::now()
                .duration_since(start)
                .expect("Time went backwards");
            println!("{:?}", since_the_epoch);
        //});

        //CPU_TEST.as_mut().unwrap().stop();
        //handle.join();
    }
}

#[test]
fn calculator4() {

    do_it();
    if true {return;}

    let file = std::fs::read_to_string("test2.cl").expect("bruh");

    let tokenizer = Lexer::new(Tokenizer::new(&file));

    match gen_parser::ProgramParser::new().parse(tokenizer) {
        Ok(val) => {
            let mut test = PrintVisitor::new();
            val.accept(Box::new(&mut test));
            println!("{}", val);
        }
        Err(val) => {
            println!("{:?}", val);
        }
    }

    //assert_eq!(&format!("{:?}", expr), "((22 * 44) + 66)");
}


fn main() {
    do_it();
    if true {return;}

    let file = std::fs::read_to_string("test2.cl").expect("bruh");
    println!("\nPrinting test.cl");

    let mut tokenizer = Tokenizer::new(&file);
    let tmp = tokenizer.tokenize();

    for t in tmp{
        println!("token: {} string: '{}'", t, tokenizer.str_from_token(&t));
    }
    tokenizer.reset();

    /*
    let mut parser = Parser::new(tokenizer);

    let result = parser.parse();

    match result {
        Ok(ok) => {
            let mut test = PrintVisitor::new();
            ok.accept(Box::new(&mut test));
            println!("{}", ok);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
     */
}
