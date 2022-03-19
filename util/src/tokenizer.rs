use std::str::Chars;

use crate::token::Token;



#[derive(Copy, Clone, PartialEq)]
pub struct BufferIndex {
    pub index: usize,
    pub index_real: usize,
    pub line: usize,
    pub column: usize,
}

impl BufferIndex {
    pub fn new() -> Self {
        BufferIndex {
            index: 0,
            index_real: 0,
            line: 0,
            column: 0,
        }
    }
}

pub fn chars_from_u8(byte: &[u8]) -> std::str::Chars {
    std::str::from_utf8(byte).unwrap().chars()
}


// struct BaseLexer<'a, T, E, S, L: Lexer<T, E>>{
//     bytes: &'a [u8],
//     iterator: Chars<'a>,
//     iterations: usize,
//     lexer: L,

//     c:char,
//     state: S,
//     matching: bool,
//     stop_reset: bool,
//     new_token: Option<Result<Token<T>, E>>,

//     current: BufferIndex,
//     start_curr: BufferIndex,
//     last: BufferIndex,
//     escape_start: BufferIndex
// }