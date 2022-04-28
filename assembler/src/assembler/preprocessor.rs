
use crate::lexer::tokenizer::{Tokenizer, TokenType};
pub type Token = util::token::Token<TokenType>;

use std::{cell::RefCell, rc::Rc, collections::LinkedList};

use util::token::{TokenData};

use super::assembler::{Assembler, AssemblerState, FileInfo};

//-------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct PPToken{
    tok: TokenType,
    location: PPArea,
}

#[derive(Clone, Debug)]
pub struct PPArea{
    pub(crate) area: TokenData,
    pub(crate) parent: Option<Box<PPArea>>,
}

impl PPArea{
    pub fn from_t_data(data: TokenData) -> Self{
        Self{
            area: data,
            parent: Option::None,
        }
    }

    pub fn add_area(&mut self, mut new_area: TokenData){
        let PPArea{
            area,
            parent,
        } = self;

        std::mem::swap(area, &mut new_area);
        let mut p = Option::None;
        std::mem::swap(parent, &mut p);
        let p = PPArea{
            area: new_area,
            parent: p,
        };
        *parent = Option::Some(Box::new(p));
    }

    pub fn add_pparea(&mut self, mut new_area: PPArea){
        std::mem::swap(self, &mut new_area);
        let mut parent = &mut self.parent;
        while let Option::Some(p) = parent{
            parent = &mut p.parent;
        }
        *parent = Option::Some(Box::new(new_area));
    }
}
// impl Into<PPArea> for &PPToken{
//     fn into(self) -> PPArea {
//         self.to_area()
//     }
// }
impl PPToken{
    pub fn new(token: Token) -> Self{
        Self{
            tok: token.t_type,
            location: PPArea::from_t_data(token.t_data)
        }
    }
    // pub fn add_parent(&mut self, parent: Token){
    //     match &mut self.parent{
    //         Some(val) => val.add_parent(parent),
    //         None => self.parent = Option::Some(Box::new(PPToken::new(parent))),
    //     }
    // }

    // pub fn to_area(&self) -> PPArea{
    //     PPArea { 
    //         area: self.tok.t_data, 
    //         parent: match &self.parent{
    //             Some(some) => Option::Some(Box::new(some.to_area())),
    //             None => Option::None,
    //         } 
    //     }
    // }
}

//-------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub enum PreProcessedLine{
    Label(String, PPArea)
}

// impl PreProcessedLine{
//     pub fn get_area(&self) -> TokenData{
//         match self{
//             PreProcessedLine::Label(_, tok) => *tok.tok.get_token_data(),
//         }
//     }
// }

//-------------------------------------------------------------------------------------------------------------

pub struct PreProcessor{
    asm_state: Rc<RefCell<AssemblerState>>,
    token_strem: TokenStream,
    last_full_label: Option<String>,
}

impl PreProcessor{
    pub fn new(assembler: &mut Assembler, input: String) -> Result<Self, ()>{

        //let mut input_buf = String::new();
        //let _size = input.read_to_string(&mut input_buf);


        let file = assembler.asm_state().add_file(input.clone());
        let file = match file{
            Result::Ok(val) => val,
            Result::Err(err) => {
                assembler.asm_state().report_os_error(format!("Failed to load file: {} ({})", input, err));
                return Result::Err(());
            },
        };
        let mut token_strem = TokenStream::new(assembler.clone_asm_state());
        match token_strem.add_stream(FileStream::new(assembler.clone_asm_state(), file)){
            Ok(_) => {},
            Err(err) => {
                assembler.asm_state().report_preprocessor_error_no_area(err);
            },
        }

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

    fn internal_next(&mut self) -> Option<PPToken>{
        self.token_strem.next()
    }

    fn argument_next(&mut self) -> Option<PPToken>{
        loop{
            let tok = self.token_strem.next();
            match tok{
                Some(tok) => {
                    match &tok.tok{
                        TokenType::Identifier(ident) => {
                            let mut new_stream = Option::None;
                            if let Option::Some(def) = self.asm_state().get_from_scope(&ident){
                                match def{
                                    super::assembler::Define::Replacement(replace) => {
                                        let mut stream = TokenVecStream::new(replace);
                                        stream.loc = Option::Some(tok.location.clone());
                                        new_stream = Option::Some(stream);
                                    },
                                    _ => {
                                        return Option::Some(tok);
                                    }
                                }
                            }
                            if let Option::Some(new_stream) = new_stream{
                                let res = self.token_strem.add_stream(new_stream);
                                match res {
                                    Ok(_) => {},
                                    Err(err) => {
                                        self.asm_state().report_preprocessor_error(err, tok.location);
                                    },
                                }
                            }
                        }
                        _ => return Option::Some(tok),
                    }
                },
                None => return Option::None,
            }
        }
    }

    fn accept_pre_processor_statement(&mut self, ident: &String, area: PPArea){
        match ident.as_str(){
            "include" => {
                match self.argument_next(){
                    Some(arg1) => {
                        match &arg1.tok{
                            TokenType::StringLiteral(path) => {
                                let file = self.asm_state().add_file(path.clone());
                                match file{
                                    Ok(file) => {
                                        let mut stream = FileStream::new(self.asm_state.clone(), file);
                                        stream.parent = Option::Some(arg1.location.clone());
                                        let res = self.token_strem.add_stream(stream);
                                        match res{
                                            Ok(_) => {},
                                            Err(err) => {
                                                self.asm_state().report_preprocessor_error(err, area)
                                            },
                                        }
                                    },
                                    Err(err) => {
                                        self.asm_state().report_preprocessor_error(format!("Failed to open file: {} ({})", path, err), arg1.location);
                                    },
                                }
                            }
                            _ =>{
                                self.asm_state().report_preprocessor_error("Invalid arguments expected file path (i.e. \"path/to/file.asm\")", area);
                            }
                        }
                    },
                    None => {
                        self.asm_state().report_preprocessor_error("Expected filepath but found no argments", area);
                    },
                }
            }
            "define" => {
                match self.internal_next(){
                    Some(ident) => {
                        match ident.tok{
                            TokenType::Identifier(def_ident) => {
                                let mut values = Vec::new();
                                loop{
                                    match self.internal_next(){
                                        Option::Some(tok) => {
                                            match &tok.tok{
                                                TokenType::Identifier(iden) => {
                                                    if iden.eq(&def_ident){
                                                        self.asm_state().report_preprocessor_error("Cannot have identifiers with the same value as the defines identifier. This will create an infinite loop!", tok.location);
                                                    }else{
                                                        values.push(tok);
                                                    }
                                                }
                                                TokenType::NewLine => {
                                                    break;
                                                }
                                                _ => {
                                                    values.push(tok);
                                                }
                                            }
                                        }
                                        Option::None => {},
                                    }
                                }
                                if values.is_empty() {
                                    self.asm_state().put_into_scope(def_ident, super::assembler::Define::Nothing);
                                }else{
                                    self.asm_state().put_into_scope(def_ident, super::assembler::Define::Replacement(values));
                                }
                            }
                            _ => {
                                self.asm_state().report_preprocessor_error("Invalid token type expected identifier", area);
                            }
                        }
                    },
                    None => {
                        self.asm_state().report_preprocessor_error("Expected identifier but found no arguments", area);
                    },
                }
            }
            "undefine" => {

                match self.internal_next(){
                    Some(val) => {

                    },
                    None => {

                    },
                }
            }
            _ => {
                self.asm_state().report_preprocessor_error(format!("Unknown preprocessor statement: {}", ident), area);
            }
        }
    }
}

impl Iterator for PreProcessor{
    type Item = PreProcessedLine;

    fn next(&mut self) -> Option<Self::Item> {
        //let mut has_encountered_error = false;
        loop{
            match self.internal_next() {
                Some(PPToken{tok, location}) => {
                    match tok{
                        TokenType::Identifier(_ident) => {
                            loop{
                                match self.argument_next(){
                                    Option::Some(tok) => {
                                        match tok.tok{
                                            TokenType::NewLine => {
                                                break;
                                            }
                                            _ => {
                                                self.asm_state().report_preprocessor_error(format!("Unexpected token: {:?}",tok.tok), tok.location)
                                            }
                                        }
                                    }
                                    Option::None => {break;},
                                }
                            }
                        }
                        TokenType::PreProcessorStatement(ident) => {
                            self.accept_pre_processor_statement(&ident, location);
                        }
                        TokenType::Label(mut ident) => {
                            if let Option::Some('.') = ident.chars().next(){
                                if let Option::Some(last_full) = &self.last_full_label{
                                    ident = format!("{}{}", last_full, ident);
                                    return Option::Some(PreProcessedLine::Label(ident, location));
                                }else{
                                    self.asm_state().report_preprocessor_error("Found local lable with no prior full lable before (hint add label without a leading '.' before this labels definition)", location);
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
                            self.asm_state().report_preprocessor_error(format!("Unexpected token: {:?}",tok), location);
                            //has_encountered_error = true;
                        }
                    }
                },
                None => {
                    return Option::None
                },
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------

struct TokenStream{
    asm_state: Rc<RefCell<AssemblerState>>,
    internal: LinkedList<Box<dyn Iterator<Item = PPToken>>>
}

impl TokenStream{
    pub fn new(state: Rc<RefCell<AssemblerState>>) -> Self{
        Self{
            asm_state: state,
            internal: LinkedList::new(),
        }
    }

    fn add_stream(&mut self, stream: impl Iterator<Item = PPToken> + 'static) -> Result<(),&'static str>{
        if self.asm_state.borrow().settings().max_token_iterators > self.internal.len(){
            self.internal.push_front(Box::new(stream));
            Result::Ok(())
        }else{
            self.internal.clear();
            Result::Err("Failed to add token stream, max streams reached")
        }

    }
}

impl Iterator for TokenStream{
    type Item = PPToken;

    fn next(&mut self) -> Option<Self::Item> {
        loop{
            match self.internal.front_mut(){
                Some(iter) => {
                    match iter.next(){
                        Some(val) => return Option::Some(val),
                        None => self.internal.pop_front(),
                    };
                },
                None => {
                    return Option::None;
                },
            }
        }

    }
}
//-------------------------------------------------------------------------------------------------------------

struct TokenVecStream{
    tokens: Vec<PPToken>,
    loc: Option<PPArea>,
}

impl TokenVecStream{
    pub fn new(data: &Vec<PPToken>) -> Self{
        let mut tokens = Vec::new();
        for tok in data.iter().rev(){
            tokens.push(tok.clone());
        }
        Self{
            tokens,
            loc: Option::None,
        }
    }
}

impl Iterator for TokenVecStream{
    type Item = PPToken;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.pop(){
            Some(mut val) => {
                match &self.loc{
                    Some(loc) => val.location.add_pparea(loc.clone()),
                    None => {},
                }
                Option::Some(val)
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
    parent: Option<PPArea>
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
    type Item = PPToken;

    fn next(&mut self) -> Option<Self::Item> {
        loop{
            match self.tokenizer.next(){
                Some(tok) => {
                    match tok{
                        Ok(mut token) => {
                            token.get_token_data_mut().file = Option::Some(self.file_id as u16);
                            
                            let location = match &self.parent{
                                // Some(parent) => {
                                //     let mut parent = parent.clone();
                                //     parent.add_area(token.t_data);
                                //     parent
                                // },
                                _ => PPArea::from_t_data(token.t_data),
                            };

                            return Option::Some(PPToken{
                                tok: token.t_type,
                                location,
                            });
                        },
                        Err(mut err) => {
                            match &mut err.part{
                                Some(val) => {
                                    val.file = Option::Some(self.file_id as u16);
                                },
                                None => {},
                            }
                            self.state.as_ref().borrow_mut().report_tokenizer_error(err);
                        },
                    }
                },
                None => return Option::None,
            }
        }
    }
}