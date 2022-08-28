#![allow(dead_code)]
#[macro_use]
extern crate lalrpop_util;

mod gen_parsers;
mod parsing_lexer;
//use parsing_lexer::ast::PrintVisitor;
//use parsing_lexer::gen_parser;
//use parsing_lexer::lexer::Lexer;
//use parsing_lexer::tokenizer::Tokenizer;

fn test_tokenizer() {
    use parsing_lexer::tokenizer::Tokenizer;
    let file = std::fs::read_to_string("res/tests/tokenizer_test.cl").unwrap();

    let mut tokenizer = Tokenizer::from_string(&file).include_whitespace(true);
    let t = tokenizer.tokenize();
    for t in t {
        println!("token: {:?} string: {:?}", t, tokenizer.str_from_token(&t));
    }
}

fn main() {
    test_tokenizer();

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
