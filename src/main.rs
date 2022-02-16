mod parsing_lexer;
//mod mtest;

use crate::parsing_lexer::parser::{Parser, PrintVisitor, TreeNode};
use crate::parsing_lexer::tokenizer::Tokenizer;


fn main() {
    let file = std::fs::read_to_string("test2.cl").expect("bruh");
    println!("\nPrinting test.cl");

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
            let mut test = PrintVisitor::new();
            ok.accept(Box::new(&mut test));
            println!("{}", ok);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
