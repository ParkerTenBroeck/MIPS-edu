
use crate::lexer::tokenizer::{Tokenizer, TokenType};
pub type Token = util::token::Token<TokenType>;

use std::{cell::RefCell, rc::Rc, error::Error, fs::File, collections::LinkedList};

use util::token::{TokenData};

use super::assembler::{Assembler, AssemblerState, FileInfo};

#[derive(Clone, Debug)]
pub struct PPToken{
    tok: Token,
    parent: Option<Box<PPToken>>,
}

//-------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum PreProcessedLine{
    Label(String, PPToken)
}

impl PreProcessedLine{
    pub fn get_area(&self) -> TokenData{
        match self{
            PreProcessedLine::Label(_, tok) => *tok.tok.get_token_data(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------

pub struct PreProcessor{
    asm_state: Rc<RefCell<AssemblerState>>,
    token_strem: TokenStream,
    last_full_label: Option<String>,
}

impl PreProcessor{
    pub fn new(assembler: &mut Assembler, input: String) -> Result<Self, Box<dyn Error>>{

        //let mut input_buf = String::new();
        //let _size = input.read_to_string(&mut input_buf);


        let file = assembler.asm_state().add_file(input)?;
        let mut token_strem = TokenStream::new();
        token_strem.add_stream(FileStream::new(assembler.clone_asm_state(), file));

        let new = Self{
            asm_state: assembler.clone_asm_state(),
            last_full_label: Option::None,
            token_strem,
        };

        Result::Ok(new)
    }

    fn asm_state(&mut self) -> std::cell::RefMut<AssemblerState>{
        self.asm_state.borrow_mut()
    }


    fn linafy(&mut self, tokenizer: &mut Tokenizer) -> Vec<Vec<Token>>{
        let mut lines = Vec::new();
    
        let mut line = Vec::new();
        for token in tokenizer{

            match token {
                Ok(token) => {
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
                Err(err) => {
                    self.asm_state().report_tokenizer_error(err);
                },
            }
            

        }
        if line.len() > 0 {
            lines.push(line);
        }
        lines
    }

    fn internal_next(&mut self) -> Option<Result<PPToken, ()>>{
        self.token_strem.next()
    }
}

impl Iterator for PreProcessor{
    type Item = Result<PreProcessedLine, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut has_encountered_error = false;
        loop{
            match self.internal_next() {
                Some(token) => {
                    match token{
                        Ok(mut token) => {
                            match token.tok.get_token_type_mut(){
                                TokenType::Identifier(ident) => {
                                   
                                }
                                TokenType::PreProcessorStatement(ident) => {

                                }
                                TokenType::Label(ident) => {
                                    if let Option::Some('.') = ident.chars().next(){
                                        if let Option::Some(last_full) = &self.last_full_label{
                                            *ident = format!("{}{}", last_full, ident);
                                            return Option::Some(Result::Ok(PreProcessedLine::Label(ident.clone(), token)));
                                        }else{
                                            self.asm_state().report_preprocessor_error("Found local lable with no prior full lable before (hint add label without a leading '.' before this labels definition)", token.tok.t_data);
                                            return Option::Some(Result::Err(()))
                                        }
                                    }else{
                                        self.last_full_label = Option::Some(ident.clone())
                                    }
                                }
                                TokenType::NewLine 
                                | TokenType::Whitespace 
                                | TokenType::Comment(_) 
                                | TokenType::InnerDocumentation(_) 
                                | TokenType::OuterDocumentation(_) => {
                                    //we can ignore these if theyre not apart of structures
                                }
                                _ => {
                                    self.asm_state().report_preprocessor_error("Invalid token found", token.tok.t_data);
                                    has_encountered_error = true;
                                }
                            }
                        },
                        Err(_) => {
                            has_encountered_error = true;
                        },
                    }
                },
                None => {
                    if has_encountered_error {
                        return Option::Some(Result::Err(()))
                    }
                    return Option::None
                },
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------

struct TokenStream{
    internal: LinkedList<Box<dyn Iterator<Item = Result<PPToken, ()>>>>
}

impl TokenStream{
    pub fn new() -> Self{
        Self{
            internal: LinkedList::new(),
        }
    }

    fn add_stream(&mut self, stream: impl Iterator<Item = Result<PPToken, ()>> + 'static){
        self.internal.push_front(Box::new(stream));
    }
}

impl Iterator for TokenStream{
    type Item = Result<PPToken, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.internal.front_mut(){
            Some(iter) => {
                return iter.next()
            },
            None => Option::None,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------

struct FileStream{
    file_id: usize,
    tokenizer: Tokenizer<'static>,
    state: Rc<RefCell<AssemblerState>>,
    parent: Option<PPToken>
}

impl FileStream{
    pub fn new(state: Rc<RefCell<AssemblerState>>, file: (usize, Rc<FileInfo>)) -> Self{
        let str = file.1.data.as_str();
        let str: &'static str = unsafe{std::mem::transmute(str)};

        let tokenizer = Tokenizer::<'static>::from_str(str)
            .include_comments(false)
            .include_documentation(false)
            .include_whitespace(false);

        Self{
            file_id: file.0,
            tokenizer,
            state,
            parent: Option::None,
        }
    }
}

impl Iterator for FileStream{
    type Item = Result<PPToken, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.next(){
            Some(tok) => {
                Option::Some(match tok{
                    Ok(mut token) => {
                        token.get_token_data_mut().file = Option::Some(self.file_id as u16);
                        Result::Ok(PPToken{
                            tok: token,
                            parent: None,
                        })
                    },
                    Err(err) => {
                        self.state.as_ref().borrow_mut().report_tokenizer_error(err);
                        Result::Err(())
                    },
                })
            },
            None => Option::None,
        }
    }
}