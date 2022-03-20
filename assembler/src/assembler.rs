use std::{fs::File, error::Error};

use crate::lexer::tokenizer::{Tokenizer, TokenType};
type Token = util::token::Token<TokenType>;


#[allow(dead_code)]
#[allow(unused)]
pub fn assemble(input: &mut File, output:&mut  File) -> Result<(), Box<dyn Error>>{
    let mut input_buf = String::new();
    let input_buf =  std::fs::read_to_string("./assembler/res/snake.asm")?;

    //let _size = input.read_to_string(&mut input_buf)?;
    let mut tokenizer = Tokenizer::from_string(&input_buf)
        .include_comments(false)
        .include_documentation(false)
        .include_whitespace(false);
    let lines = linafy(&mut tokenizer);
    
    for line in lines{

        //if line[0].get_token_data().get_line() > 30{
        //    break;
        //}

        print!("{}: ", line[0].get_token_data().get_line() + 1);

        for token in line{
            print!("{:?}  ", token.get_token_type());
        }
        println!();
        
    }
    
    let mut start = true;

    Ok(())
}

fn linafy(tokenizer: &mut Tokenizer) -> Vec<Vec<Token>>{
    let mut lines = Vec::new();

    let mut line = Vec::new();
    for token in tokenizer{
        
        use crate::lexer::tokenizer::TokenType::*;
        match token.get_token_type(){
            NewLine => {
                if line.len() > 0 {
                    lines.push(line);
                }
                line = Vec::new();
            }
            _ => {
                line.push(token);
            }
        }
    }
    lines
}