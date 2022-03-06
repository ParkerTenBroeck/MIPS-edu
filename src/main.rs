#[macro_use] extern crate lalrpop_util;
mod parsing_lexer;
mod virtual_cpu;


use parsing_lexer::ast::*;
use parsing_lexer::gen_parser;
use parsing_lexer::lexer::Lexer;
use parsing_lexer::tokenizer::Tokenizer;


#[test]
fn calculator4() {
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
