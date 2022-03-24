use std::{fs::File, error::Error, io::Read, collections::HashMap, ops::DerefMut};

use crate::lexer::tokenizer::{Tokenizer, TokenType};
type Token = util::token::Token<TokenType>;

#[derive(Clone)]
struct PPToken{
    tok: Token,
    parent: Option<Box<PPToken>>,
}

#[allow(dead_code)]
#[allow(unused)]
pub fn assemble(input: &mut File, output:&mut  File) -> Result<(), Box<dyn Error>>{
    Assembler::new().assemble(input, output)
}


enum Define{
    Replacement(Vec<PPToken>),
    Macro(()),
    Label(String),
    Nothing,
}

#[derive(Default)]
struct Scope{
    values: HashMap<String, Define>,
}

impl Scope {
    fn new() -> Self {
        Scope{
            ..Default::default()
        }
    }
}

#[derive(Default)]
struct Assembler{
    scope: Vec<Scope>,
}

impl Assembler{

    pub fn new() -> Self{
        Assembler {
            ..Default::default()
        }
    }

    fn get_from_scope<'a>(&'a mut self, ident: &String) -> Option<&'a mut Define>{
        

        for scope in self.scope.iter_mut().rev(){
            match scope.values.get_mut(ident){
                Some(val) => {
                    return Option::Some(val);
                },
                None => continue,
            }
        }

        Option::None
        // let mut i = self.scope.len() as i32;
        // while {i -= 1; i >= 0} 
        // {
        //     let scope: Option<&'a mut Scope> = self.scope.get_mut(i as usize);
        //     match scope {
        //         Some(val) => {
        //             match val.values.get_mut(ident){
        //                 Some(val) => {
        //                     return Option::Some(val);
        //                 },
        //                 None => continue,
        //             }
        //         },
        //         None => continue,
        //     }
            
        // }
        // Option::None
    }

    fn put_into_scope(&mut self, ident: String, val: Define) {
        let mut len = self.scope.len();
        if len == 0 {
            self.scope.push(Scope::new());
            len = 1;
        }
        self.scope[len - 1].values.insert(ident, val);
    }

    #[allow(dead_code)]
    #[allow(unused)]
    pub fn assemble(&mut self, input: &mut File, output:&mut  File) -> Result<(), Box<dyn Error>>{
        let mut input_buf = String::new();

        //let input_buf =  std::fs::read_to_string("./assembler/res/snake.asm")?;
        let size = input.read_to_string(&mut input_buf)?;
        
        let mut tokenizer = Tokenizer::from_string(&input_buf)
            .include_comments(false)
            .include_documentation(false)
            .include_whitespace(false);
        let lines = Self::linafy(&mut tokenizer);
        
        let mut lines = self.preprocess(lines);

        {
            let mut last_label = Option::<&str>::None;
            for line in &mut lines{
                't_loop:
                for token in line{
                    match token.tok.get_token_type_mut() {
                        TokenType::Label(val) => {
                            if &val.as_str()[..1] == "."{
                                if let Option::Some(la) = last_label{
                                    val.insert_str(0, la);
                                }else{
                                    
                                    //token.tok.t_type = TokenType::ERROR(format!(""));
                                    //error handleing
                                    todo!();
                                    continue 't_loop;
                                }
                            }else{
                                last_label = Option::Some(val.as_str());
                            }
                        }
                        TokenType::Identifier(val) => {
                            if &val.as_str()[..1] == "."{
                                if let Option::Some(la) = last_label{
                                    val.insert_str(0, la);
                                }else{
                                    //error handleing
                                    todo!();
                                }
                            }
                        }
                        _ => {
    
                        }
                    }
                }
            }    
        }

        for line in lines{

            if line[0].tok.get_line() > 10 {
                //break;
            }
    
            print!("{}: ", line[0].tok.get_token_data().get_line() + 1);

            for token in line{
                print!("{:?}  ", token.tok.get_token_type());
            }
            println!();
        }   
        let mut start = true;

        Ok(())
    }

    fn actually_assemble(&mut self){
        
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
        if line.len() > 0 {
            lines.push(line);
        }
        lines
    }

    fn accept_preprocess_statement(&mut self, line: &mut Vec<Token>) -> Option<Vec<PPToken>>{
        let mut first = line.remove(0);
        if let TokenType::PreProcessorStatement(ident) = first.t_type{
            match ident.as_str(){
                "define" => {
                    let second = line.remove(0);
                    if let TokenType::Identifier(ident) = second.t_type{
                        if line.len() > 0{
                            let mut pp_line = Vec::new();
                            while line.len() > 0{
                                let tok = line.remove(0);
                                pp_line.push(
                                    PPToken{
                                    tok,
                                    parent: Option::None,
                                });
                            }
                            self.put_into_scope(ident, Define::Replacement(pp_line));
                        }else{
                            self.put_into_scope(ident, Define::Nothing);
                        }
                    }else{
                        let mut new: Vec<PPToken> = Vec::new();
                        first.t_type = TokenType::ERROR(format!("Only a valid identifier can be used in define"));
                        let first = PPToken { tok: first, parent: Option::None };
                        new.push(first);
                        return Option::Some(new);
                    }
                },
                "undefine" => {

                },
                _ => {
                    let mut new: Vec<PPToken> = Vec::new();
                    first.t_type = TokenType::ERROR(format!("how did this happen??"));
                    let first = PPToken { tok: first, parent: Option::None };
                    new.push(first);
                    return Option::Some(new);
                }
            }
        }
        Option::None
    }
    

    fn preprocess(&mut self, data: Vec<Vec<Token>>) -> Vec<Vec<PPToken>>{
        let mut lines = Vec::new();

        for mut line in data{
            let len = line.len();
            if len >= 1{
                match line[0].get_token_type() {
                    TokenType::PreProcessorStatement(_) =>{
                        match self.accept_preprocess_statement(&mut line){
                            Some(val) => {
                                lines.push(val);
                            },
                            None => {},
                        }
                    }
                    _ => {
                        let mut n_line = Vec::new();
                        for tok in line{
                            match &tok.t_type {
                                TokenType::Identifier(val) => {
                                    match self.get_from_scope(val) {
                                        Some(val) => {
                                            match val{
                                                Define::Replacement(val) => {
                                                    for tok_i in val{
                                                        let tok_i = tok_i.clone();
                                                        //tok_i.parent = Box::new(tok.clone());
                                                        n_line.push(tok_i);   
                                                    }
                                                },
                                                Define::Label(_) => todo!(),
                                                Define::Macro(_) | Define::Nothing => {
                                                    n_line.push(PPToken{
                                                        tok,
                                                        parent: Option::None,
                                                    })   
                                                },
                                            }
                                        },
                                        None => {
                                            n_line.push(PPToken{
                                                tok,
                                                parent: Option::None,
                                            })
                                        },
                                    }
                                }
                                _ => {
                                    n_line.push(PPToken{
                                        tok,
                                        parent: Option::None,
                                    })
                                }
                            }
                        }
                        lines.push(n_line);
                    }
                }
            }
        }

        lines
    }
}