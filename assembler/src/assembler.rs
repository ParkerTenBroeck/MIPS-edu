use std::{fs::File, io::Read, error::Error};
use crate::lexer::tokenizer::Tokenizer;

#[allow(dead_code)]
fn assemble(input: &mut File, output:&mut  File) -> Result<(), Box<dyn Error>>{
    let mut input_buf = String::new();
    input.read_to_string(&mut input_buf)?;
    let mut tokenizer = Tokenizer::from_string(&input_buf);
    let tokens = tokenizer.tokenize();
    
    let mut start = true;
    for t in tokens{
        match t.get_token_type(){
            crate::lexer::tokenizer::TokenType::LPar => todo!(),
            crate::lexer::tokenizer::TokenType::RPar => todo!(),
            crate::lexer::tokenizer::TokenType::LBrace => todo!(),
            crate::lexer::tokenizer::TokenType::RBrace => todo!(),
            crate::lexer::tokenizer::TokenType::LBracket => todo!(),
            crate::lexer::tokenizer::TokenType::RBracket => todo!(),
            crate::lexer::tokenizer::TokenType::Plus => todo!(),
            crate::lexer::tokenizer::TokenType::Minus => todo!(),
            crate::lexer::tokenizer::TokenType::Star => todo!(),
            crate::lexer::tokenizer::TokenType::Slash => todo!(),
            crate::lexer::tokenizer::TokenType::Ampersand => todo!(),
            crate::lexer::tokenizer::TokenType::BitwiseOr => todo!(),
            crate::lexer::tokenizer::TokenType::BitwiseXor => todo!(),
            crate::lexer::tokenizer::TokenType::BitwiseNot => todo!(),
            crate::lexer::tokenizer::TokenType::ShiftLeft => todo!(),
            crate::lexer::tokenizer::TokenType::ShiftRight => todo!(),
            crate::lexer::tokenizer::TokenType::Percent => todo!(),
            crate::lexer::tokenizer::TokenType::LogicalAnd => todo!(),
            crate::lexer::tokenizer::TokenType::LogicalOr => todo!(),
            crate::lexer::tokenizer::TokenType::LogicalNot => todo!(),
            crate::lexer::tokenizer::TokenType::Dot => todo!(),
            crate::lexer::tokenizer::TokenType::Comma => todo!(),
            crate::lexer::tokenizer::TokenType::Colon => todo!(),
            crate::lexer::tokenizer::TokenType::Semicolon => todo!(),
            crate::lexer::tokenizer::TokenType::QuestionMark => todo!(),
            crate::lexer::tokenizer::TokenType::At => todo!(),
            crate::lexer::tokenizer::TokenType::Octothorp => todo!(),
            crate::lexer::tokenizer::TokenType::Dollar => todo!(),
            crate::lexer::tokenizer::TokenType::LessThan => todo!(),
            crate::lexer::tokenizer::TokenType::LessThanEq => todo!(),
            crate::lexer::tokenizer::TokenType::GreaterThan => todo!(),
            crate::lexer::tokenizer::TokenType::GreaterThanEq => todo!(),
            crate::lexer::tokenizer::TokenType::Equals => todo!(),
            crate::lexer::tokenizer::TokenType::NotEquals => todo!(),
            crate::lexer::tokenizer::TokenType::Assignment => todo!(),
            crate::lexer::tokenizer::TokenType::StringLiteral(_) => todo!(),
            crate::lexer::tokenizer::TokenType::I8Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::I16Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::I32Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::I64Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::I128Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::U8Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::U16Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::U32Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::U64Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::U128Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::F32Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::F64Literal(_) => todo!(),
            crate::lexer::tokenizer::TokenType::CharLiteral(_) => todo!(),
            crate::lexer::tokenizer::TokenType::BoolLiteral(_) => todo!(),
            crate::lexer::tokenizer::TokenType::Identifier(_) => todo!(),
            crate::lexer::tokenizer::TokenType::NewLine => todo!(),
            _ => {
                panic!("unexpected token{:?}", t);
            }
        }
    }

    Ok(())
}