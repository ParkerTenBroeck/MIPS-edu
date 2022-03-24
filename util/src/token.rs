
#[derive(Debug, Clone)]
pub struct Token<T>{
    pub t_type: T,
    pub t_data: TokenData,
}

#[allow(unused)]
impl<T> Token<T> {
    pub fn get_token_type(&self) -> &T { &self.t_type }
    pub fn get_token_type_mut(&mut self) -> &mut T { &mut self.t_type }
    
    pub fn get_token_data(&self) -> &TokenData { &self.t_data }
    pub fn get_real_size(&self) -> usize {
        self.t_data.size_real
    }
    pub fn get_real_index(&self) -> usize {
        self.t_data.index_real
    }
    pub fn get_size(&self) -> usize {
        self.t_data.size
    }
    pub fn get_index(&self) -> usize {
        self.t_data.index
    }
    pub fn get_line(&self) -> usize {
        self.t_data.line
    }
    pub fn get_column(&self) -> usize {
        self.t_data.column
    }
}

use std::fmt::Formatter;
use std::fmt::Display;
use std::str::FromStr;

impl<T: Display> Display for Token<T>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "line:{}, size:{}, column:{} type: {}",self.get_line() + 1, self.get_size(),self.get_column(), self.t_type)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TokenData {
    pub size: usize,
    pub index: usize,
    pub index_real:usize,
    pub size_real:usize,
    pub line: usize,
    pub column: usize
}

#[allow(unused)]
impl TokenData {
    pub fn new()-> Self{
        TokenData{
            size: 0,
            index: 0,
            index_real: 0,
            size_real: 0,
            line: 0,
            column: 0,
        }
    }
    pub fn get_real_size(&self) -> usize {
        self.size_real
    }
    pub fn get_real_index(&self) -> usize {
        self.index_real
    }
    pub fn get_size(&self) -> usize {
        self.size
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn get_line(&self) -> usize {
        self.line
    }
    pub fn get_column(&self) -> usize {
        self.column
    }

    pub fn string_from_token(&self, source: &str) -> String {
        String::from_str(self.str_from_token(source)).unwrap()
    }
    pub fn str_from_token<'a>(&self, source: &'a str) -> &'a str {
        &source[self.get_real_index()..self.get_real_index() + self.get_real_size()]
    }
}