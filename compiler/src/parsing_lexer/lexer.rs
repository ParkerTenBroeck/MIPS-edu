use crate::parsing_lexer::tokenizer::{Token, TokenType, Tokenizer};

pub struct Lexer<'input> {
    tokenizer: Tokenizer<'input>,
}

impl<'input> Lexer<'input> {
    pub fn new(tokenizer: Tokenizer<'input>) -> Self {
        Lexer { tokenizer }
    }
}

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokenizer.next() {
            None => None,
            Some(
                val @ Token {
                    t_type: TokenType::Error(_),
                    ..
                },
            ) => Option::Some(Err(val)),
            Some(val) => {
                let size = val.get_real_size();
                Option::Some(Ok((val.get_real_index(), val, size)))
            }
        }
    }
}
