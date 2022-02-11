use crate::parser::Parser;
use crate::tokenizer::Tokenizer;

mod tokenizer;
mod parser;

fn main() {
    let file = std::fs::read_to_string("test2.cl").expect("bruh");
    println!("\nPrinting test.cl");

    let mut tokenizer = Tokenizer::new(&file);
    //let tmp = tokenizer.tokenize();

    //for t in tmp{
    //    println!("token: {} string: '{}'", t, tokenizer.str_from_token(&t));
    //}
    //tokenizer.reset();

    let mut parser = Parser::new(tokenizer);

    let result = parser.parse();

    println!("{:?}", result);
}
