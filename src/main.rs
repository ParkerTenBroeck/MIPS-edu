use crate::tokenizer::{Tokenizer};

mod tokenizer;

fn main() {
    let file = std::fs::read_to_string("test.cl").expect("bruh");
    println!("\nPrinting test.cl");

    let tmp = Tokenizer::new(&file).tokenize();
    for t in tmp{
        //println!("{}", t);
    }
}
