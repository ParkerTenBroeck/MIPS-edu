#[derive(Debug, Clone)]
pub struct Token<T> {
    pub t_type: T,
    pub t_data: TokenData,
}

#[allow(unused)]
impl<T> Token<T> {
    pub fn get_token_type(&self) -> &T {
        &self.t_type
    }
    pub fn get_token_type_mut(&mut self) -> &mut T {
        &mut self.t_type
    }

    pub fn get_token_data(&self) -> &TokenData {
        &self.t_data
    }
    pub fn get_token_data_mut(&mut self) -> &mut TokenData {
        &mut self.t_data
    }
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
    pub fn get_file(&self) -> Option<u16> {
        self.t_data.file
    }
}

use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

impl<T: Display> Display for Token<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line:{}, size:{}, column:{} type: {}",
            self.get_line() + 1,
            self.get_size(),
            self.get_column(),
            self.t_type
        )
    }
}

#[derive(Debug)]
pub struct TokenizerError {
    pub part: Option<TokenData>,
    pub error: String,
}
impl TokenizerError {
    pub fn at_pos(error: String, pos: TokenData) -> Self {
        Self {
            error,
            part: Option::Some(pos),
        }
    }
    pub fn error(error: String) -> Self {
        Self {
            error,
            part: Option::None,
        }
    }
}

impl std::error::Error for TokenizerError {}
impl std::fmt::Display for TokenizerError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Copy, Clone, PartialEq, Default, Eq, Debug)]
pub struct TokenData {
    pub size: usize,
    pub index: usize,
    pub index_real: usize,
    pub size_real: usize,
    pub line: usize,
    pub column: usize,
    pub file: Option<u16>,
}

#[allow(unused)]
impl TokenData {
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
    pub fn get_file(&self) -> Option<u16> {
        self.file
    }

    pub fn string_from_token(&self, source: &str) -> String {
        String::from_str(self.str_from_token(source)).unwrap()
    }
    pub fn str_from_token<'a>(&self, source: &'a str) -> &'a str {
        &source[self.get_real_index()..self.get_real_index() + self.get_real_size()]
    }
}
