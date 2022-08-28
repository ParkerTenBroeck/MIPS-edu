use std::mem;
use std::str::Chars;

use util::token::TokenizerError;

pub type Token = util::token::Token<TokenType>;
pub type TokenData = util::token::TokenData;
pub type BufferIndex = util::tokenizer::BufferIndex;

//test
#[derive(Debug, Clone)]
pub enum TokenType {
    LPar,
    RPar,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    ShiftLeft,
    ShiftRight,
    Percent,
    LogicalAnd,
    LogicalOr,
    LogicalNot,

    Dot,
    Comma,
    Colon,
    Semicolon,
    QuestionMark,
    At,
    Octothorp,
    Dollar,

    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equals,
    NotEquals,

    Assignment,

    StringLiteral(String),
    I8Literal(i8),
    I16Literal(i16),
    I32Literal(i32),
    I64Literal(i64),
    I128Literal(i128),
    U8Literal(u8),
    U16Literal(u16),
    U32Literal(u32),
    U64Literal(u64),
    U128Literal(u128),
    F32Literal(f32),
    F64Literal(f64),
    CharLiteral(char),
    BoolLiteral(bool),
    Identifier(String),
    Directive(String),
    PreProcessorStatement(String),
    Label(String),
    Register(u32),

    Whitespace,
    NewLine,

    Comment(String),
    OuterDocumentation(String),
    InnerDocumentation(String),
    //ERROR(String)
}

type TokenizerItem = std::option::Option<std::result::Result<Token, TokenizerError>>;

trait IntoWithData<T>: Sized {
    /// Performs the conversion.
    fn into_w_data(self, _: TokenData) -> T;
}

impl IntoWithData<TokenizerItem> for TokenType {
    fn into_w_data(self, t_data: TokenData) -> TokenizerItem {
        Option::Some(Result::Ok(Token {
            t_data,
            t_type: self,
        }))
    }
}

impl IntoWithData<TokenizerItem> for String {
    fn into_w_data(self, data: TokenData) -> TokenizerItem {
        Option::Some(Result::Err(TokenizerError::at_pos(self, data)))
    }
}

impl IntoWithData<TokenizerItem> for &str {
    fn into_w_data(self, data: TokenData) -> TokenizerItem {
        Option::Some(Result::Err(TokenizerError::at_pos(self.into(), data)))
    }
}

pub enum IdentifierMode {
    Ascii,
    Unicode,
}

pub struct Tokenizer<'a> {
    bytes: &'a [u8],
    iterator: Chars<'a>,
    iterations: usize,

    c: char,
    state: State,
    matching: bool,
    stop_reset: bool,
    new_token: TokenizerItem,

    current: BufferIndex,
    start_curr: BufferIndex,
    last: BufferIndex,
    escape_start: BufferIndex,

    escape_error: bool,
    string: String,
    char_literal: char,

    ident_mode: IdentifierMode,
    include_whitespace: bool,
    include_comments: bool,
    include_documentation: bool,
}

#[derive(Debug)]
enum State {
    Default,

    NumberLiteral(i32),

    RawIdentifierStart,
    IdentifierContinue,

    StringStart,
    StringNormal,
    StringEscape,
    CharLiteralStart,
    CharLiteralNormal(),
    CharLiteralEscape(),

    AssemblyComment,
    LineComment(bool),
    BlockComment(u8, u32),

    OuterDoc(bool),
    OuterBlockDoc(u8, u32),
    InnerDoc,
    InnerBlockDoc(u8, u32),

    Colon,
    Equal,
    Minus,
    Bang,
    LessThan,
    GreaterThan,
    And,
    Or,
    Xor,
    Plus,
    Star,
    Div,
    Mod,
    ShiftLeft,
    ShiftRight,
    EscapeCharacter(Box<State>, i32),

    Whitespace,
    Backslash,

    EOF,
}

impl Iterator for Tokenizer<'_> {
    type Item = Result<Token, TokenizerError>;

    fn next(&mut self) -> TokenizerItem {
        loop {
            match (&self.state, self.c) {
                (State::EOF, '\0') => {
                    return Option::None;
                }
                (State::Default, '\0') => {}
                (_, '\0') => {
                    self.matching = true;
                    let tmp =
                        self.create_token(format!("Reached EOF while in state: {:?}", self.state));
                    self.state = State::Default;
                    self.matching = false;
                    return tmp;
                }
                _ => {}
            }

            if !self.matching {
                match self.iterator.next() {
                    Some(char) => {
                        self.c = char;
                        self.iterations += 1;
                        self.ntm();
                    }
                    None => match self.state {
                        State::Default => {
                            return Option::None;
                        }
                        _ => {
                            self.c = '\0';
                            self.iterations += 1;
                            self.ntm();
                        }
                    },
                }
                if self.c == '\n' {
                    self.current.line += 1;
                    self.current.column = 0;
                }

                self.matching = true;
            }

            while self.matching {
                self.matching = false;
                match &self.state {
                    State::Default => {
                        match self.c {
                            '\0' => {
                                self.state = State::EOF;
                            }

                            ' ' | '\t' | '\r' => {
                                self.state = State::Whitespace;
                            }
                            '\n' => {
                                self.new_token = self.create_token(TokenType::NewLine);
                            }
                            '\\' => {
                                self.state = State::Backslash;
                            }

                            '|' => self.state = State::Or,
                            '^' => self.state = State::Xor,
                            '/' => self.state = State::Div,
                            '%' => self.state = State::Mod,
                            '-' => self.state = State::Minus,
                            '+' => self.state = State::Plus,
                            '*' => self.state = State::Star,
                            '=' => self.state = State::Equal,

                            '"' => self.state = State::StringStart,
                            '\'' => self.state = State::CharLiteralStart,
                            //'.' => self.state = State::Dot(0),
                            '<' => self.state = State::LessThan,
                            '>' => self.state = State::GreaterThan,
                            '!' => self.state = State::Bang,
                            '&' => self.state = State::And,

                            ':' => self.state = State::Colon,

                            '(' => self.new_token = self.create_token(TokenType::LPar),
                            ')' => self.new_token = self.create_token(TokenType::RPar),
                            '{' => self.new_token = self.create_token(TokenType::LBrace),
                            '}' => self.new_token = self.create_token(TokenType::RBrace),
                            '[' => self.new_token = self.create_token(TokenType::LBracket),
                            ']' => self.new_token = self.create_token(TokenType::RBracket),
                            ';' => self.state = State::AssemblyComment,
                            '~' => self.new_token = self.create_token(TokenType::BitwiseNot),
                            ',' => self.new_token = self.create_token(TokenType::Comma),
                            '?' => self.new_token = self.create_token(TokenType::QuestionMark),
                            '@' => self.new_token = self.create_token(TokenType::At),

                            '#' => self.state = State::RawIdentifierStart,
                            '$' => self.state = State::RawIdentifierStart,

                            '0'..='9' => self.state = State::NumberLiteral(0),

                            _ => {
                                if self.is_curr_ident_start() {
                                    self.state = State::IdentifierContinue;
                                } else if self.c.is_whitespace() {
                                    self.state = State::Whitespace;
                                } else {
                                    let message = format!("Unexpected Char: {:?}", self.c);
                                    self.new_token = self.create_token(message);
                                }
                            }
                        }
                    }
                    State::Backslash => match self.c {
                        '\r' => {
                            self.state = State::Backslash;
                        }
                        '\n' => {
                            self.state = State::Default;
                        }
                        _ => {
                            self.new_token = self.create_token(format!(
                                "illegal character after \\ {:?} can only have \\n or \\r",
                                self.c
                            ));
                            self.state = State::Default;
                        }
                    },
                    State::AssemblyComment => match self.c {
                        '\n' => {
                            self.matching = true;
                            self.create_token(TokenType::Comment(
                                self.curr_str().replacen(";", "", 1),
                            ));
                            self.state = State::Default;
                        }
                        _ => {
                            self.state = State::AssemblyComment;
                        }
                    },
                    State::IdentifierContinue => {
                        if self.is_curr_ident_continue() {
                            self.state = State::IdentifierContinue;
                        } else {
                            self.matching = true;
                            let ident = self.curr_str();
                            let start = ident.chars().next().unwrap();
                            match self.c {
                                ':' => {
                                    if self.is_ident_start(start) || start == '.' {
                                        let ident = self.curr_str();
                                        self.matching = false;
                                        self.new_token = self.create_token(TokenType::Label(ident)); //self.create_token(TokenType::Identifier(ident.to_string()));
                                        self.state = State::Default;
                                    } else {
                                        self.matching = true;
                                        self.new_token = self.create_token(format!(
                                            "Invalid starting character for lable: {:?}",
                                            start
                                        ));
                                        self.state = State::Default;
                                    }
                                }
                                _ => {
                                    match start {
                                        //'.' => {
                                        //    let ident = ident.replacen(".", "", 1);
                                        //    self.new_token = self.create_token(TokenType::Identifier(ident));
                                        //    self.state = State::Default;
                                        //}
                                        '#' => {
                                            let ident = ident.replacen("#", "", 1);
                                            self.new_token = self.create_token(
                                                TokenType::PreProcessorStatement(ident),
                                            );
                                            self.state = State::Default;
                                        }
                                        '$' => {
                                            let ident = ident.replacen("$", "", 1);
                                            self.new_token =
                                                self.create_identifier_or_keyword(ident); //self.create_token(TokenType::Identifier(ident.to_string()));
                                                                                          //self.new_token = self.create_token(TokenType::Identifier(ident));
                                            self.state = State::Default;
                                        }
                                        _ => {
                                            self.new_token =
                                                self.create_identifier_or_keyword(ident); //self.create_token(TokenType::Identifier(ident.to_string()));
                                            self.state = State::Default;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    State::RawIdentifierStart => {
                        if self.is_curr_ident_continue() {
                            self.state = State::IdentifierContinue;
                        } else {
                            let message = format!(
                                "Unexpected Char: {:?} ident must be one or more characters",
                                self.c
                            );
                            self.new_token = self.create_token(message);
                        }
                    }
                    State::Whitespace => match self.c {
                        ' ' | '\t' | '\r' => {
                            self.state = State::Whitespace;
                        }
                        _ => {
                            if self.c.is_whitespace() && self.c != '\n' {
                                self.state = State::Whitespace;
                            } else {
                                self.default_reset(true, TokenType::Whitespace);
                            }
                        }
                    },

                    State::BlockComment(val, rec) => {
                        let rec = *rec;
                        match *val {
                            0 => match self.c {
                                '!' => self.state = State::InnerBlockDoc(0, 0),
                                '*' => self.state = State::BlockComment(1, 0),
                                _ => self.state = State::BlockComment(10, rec),
                            },
                            1 => match self.c {
                                '/' => {
                                    self.new_token =
                                        self.create_token(TokenType::Comment("".into()));
                                    self.state = State::Default;
                                }
                                '*' => self.state = State::BlockComment(2, rec),
                                _ => self.state = State::OuterBlockDoc(0, 0),
                            },
                            2 => match self.c {
                                '/' => {
                                    self.new_token =
                                        self.create_token(TokenType::Comment("".into()));
                                    self.state = State::Default;
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::BlockComment(10, rec)
                                }
                            },
                            10 => match self.c {
                                '/' => self.state = State::BlockComment(12, rec),
                                '*' => self.state = State::BlockComment(11, rec),
                                _ => self.state = State::BlockComment(10, rec),
                            },
                            11 => match self.c {
                                '/' => {
                                    if rec == 0 {
                                        let comment = self.curr_str().replacen("/*", "", 1);
                                        self.new_token =
                                            self.create_token(TokenType::Comment(comment));
                                        self.state = State::Default;
                                    } else {
                                        self.state = State::BlockComment(10, rec - 1);
                                    }
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::BlockComment(10, rec);
                                }
                            },
                            12 => match self.c {
                                '*' => {
                                    self.state = State::BlockComment(10, rec + 1);
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::BlockComment(10, rec);
                                }
                            },
                            _ => {}
                        }
                    }
                    State::OuterBlockDoc(val, rec) => {
                        let rec = *rec;
                        match *val {
                            0 => match self.c {
                                '/' => self.state = State::OuterBlockDoc(2, rec),
                                '*' => self.state = State::OuterBlockDoc(1, rec),
                                _ => self.state = State::OuterBlockDoc(0, rec),
                            },
                            1 => match self.c {
                                '/' => {
                                    if rec == 0 {
                                        let comment = self.curr_str().replacen("/**", "", 1);
                                        self.new_token = self
                                            .create_token(TokenType::OuterDocumentation(comment));
                                        self.state = State::Default;
                                    } else {
                                        self.state = State::OuterBlockDoc(0, rec - 1);
                                    }
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::OuterBlockDoc(0, rec);
                                }
                            },
                            2 => match self.c {
                                '*' => {
                                    self.state = State::OuterBlockDoc(0, rec + 1);
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::OuterBlockDoc(0, rec);
                                }
                            },
                            _ => {}
                        }
                    }
                    State::InnerBlockDoc(val, rec) => {
                        let rec = *rec;
                        match *val {
                            0 => match self.c {
                                '/' => self.state = State::InnerBlockDoc(2, rec),
                                '*' => self.state = State::InnerBlockDoc(1, rec),
                                _ => self.state = State::InnerBlockDoc(0, rec),
                            },
                            1 => match self.c {
                                '/' => {
                                    if rec == 0 {
                                        let comment = self.curr_str().replacen("/**", "", 1);
                                        self.new_token = self
                                            .create_token(TokenType::InnerDocumentation(comment));
                                        self.state = State::Default;
                                    } else {
                                        self.state = State::InnerBlockDoc(0, rec - 1);
                                    }
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::InnerBlockDoc(0, rec);
                                }
                            },
                            2 => match self.c {
                                '*' => {
                                    self.state = State::InnerBlockDoc(0, rec + 1);
                                }
                                _ => {
                                    self.matching = true;
                                    self.state = State::InnerBlockDoc(0, rec);
                                }
                            },
                            _ => {}
                        }
                    }
                    State::InnerDoc => match self.c {
                        '\n' | '\r' | '\0' => {
                            self.matching = true;
                            let comment = self.curr_str().replacen("//!", "", 1);
                            self.new_token =
                                self.create_token(TokenType::InnerDocumentation(comment));
                            self.state = State::Default;
                        }
                        _ => {
                            self.state = State::InnerDoc;
                        }
                    },
                    State::OuterDoc(val) => match self.c {
                        '\n' | '\r' | '\0' => {
                            self.matching = true;
                            let comment = self.curr_str().replacen("///", "", 1);
                            self.new_token =
                                self.create_token(TokenType::OuterDocumentation(comment));
                            self.state = State::Default;
                        }
                        '/' => {
                            if !val {
                                self.state = State::LineComment(true);
                            } else {
                                self.state = State::OuterDoc(true);
                            }
                        }
                        _ => {
                            self.state = State::OuterDoc(true);
                        }
                    },
                    State::LineComment(val) => match self.c {
                        '\n' | '\r' | '\0' => {
                            self.matching = true;
                            let comment = self.curr_str().replacen("//", "", 1);
                            self.new_token = self.create_token(TokenType::Comment(comment));
                            self.state = State::Default;
                        }
                        '/' => {
                            if !val {
                                self.state = State::OuterDoc(false);
                            } else {
                                self.state = State::LineComment(true);
                            }
                        }
                        '!' => {
                            if !val {
                                self.state = State::InnerDoc;
                            } else {
                                self.state = State::LineComment(true);
                            }
                        }
                        _ => {
                            self.state = State::LineComment(true);
                        }
                    },
                    State::Colon => match self.c {
                        _ => self.default_reset(true, TokenType::Colon),
                    },
                    State::ShiftLeft => match self.c {
                        _ => self.default_reset(true, TokenType::ShiftLeft),
                    },
                    State::ShiftRight => match self.c {
                        _ => self.default_reset(true, TokenType::ShiftRight),
                    },
                    State::GreaterThan => match self.c {
                        '>' => self.state = State::ShiftRight,
                        '=' => self.default_reset(false, TokenType::GreaterThanEq),
                        _ => self.default_reset(true, TokenType::GreaterThan),
                    },
                    State::LessThan => match self.c {
                        '<' => self.state = State::ShiftLeft,
                        '=' => self.default_reset(false, TokenType::LessThanEq),
                        _ => self.default_reset(true, TokenType::LessThan),
                    },
                    State::Bang => match self.c {
                        '=' => self.default_reset(false, TokenType::NotEquals),
                        _ => self.default_reset(true, TokenType::LogicalNot),
                    },
                    State::Plus => match self.c {
                        _ => self.default_reset(true, TokenType::Plus),
                    },
                    State::Minus => match self.c {
                        _ => self.default_reset(true, TokenType::Minus),
                    },
                    State::Star => match self.c {
                        _ => self.default_reset(true, TokenType::Star),
                    },
                    State::Div => match self.c {
                        '/' => self.state = State::LineComment(false),
                        '*' => self.state = State::BlockComment(0, 0),
                        _ => self.default_reset(true, TokenType::Slash),
                    },
                    State::Mod => match self.c {
                        _ => self.default_reset(true, TokenType::Percent),
                    },
                    State::Xor => match self.c {
                        _ => self.default_reset(true, TokenType::BitwiseXor),
                    },
                    State::Or => match self.c {
                        '|' => self.default_reset(false, TokenType::LogicalOr),
                        _ => self.default_reset(true, TokenType::BitwiseOr),
                    },
                    State::And => match self.c {
                        '&' => self.default_reset(false, TokenType::LogicalAnd),
                        _ => self.default_reset(true, TokenType::Ampersand),
                    },
                    State::EscapeCharacter(_, _) => {
                        let mut t = State::Default;
                        mem::swap(&mut self.state, &mut t);
                        let state;
                        let val;
                        match t {
                            State::EscapeCharacter(i_state, i_val) => {
                                state = *i_state;
                                val = i_val;
                            }
                            _ => {
                                panic!()
                            }
                        }
                        if val == 0 {
                            self.escape_start = self.last;
                            self.escape_error = false;
                        }
                        match (self.c, val) {
                            ('\\', 0) => {
                                self.char_literal = '\\';
                                self.state = state;
                            }
                            ('n', 0) => {
                                self.char_literal = '\n';
                                self.state = state;
                            }
                            ('r', 0) => {
                                self.char_literal = '\r';
                                self.state = state;
                            }
                            ('"', 0) => {
                                self.char_literal = '\"';
                                self.state = state;
                            }
                            ('\'', 0) => {
                                self.char_literal = '\'';
                                self.state = state;
                            }
                            ('0', 0) => {
                                self.char_literal = '\\';
                                self.state = state;
                            }
                            ('u', 0) => {
                                self.char_literal = '\0';
                                self.state = State::EscapeCharacter(Box::new(state), 1);
                            }
                            ('{', 1) => {
                                self.state = State::EscapeCharacter(Box::new(state), 2);
                            }
                            ('0'..='9' | 'a'..='f' | 'A'..='F', 2..=7) => {
                                if !self.escape_error {
                                    let tmp = char::from_u32(
                                        ((self.char_literal as u32) << 4)
                                            | u32::from_str_radix(self.c.to_string().as_str(), 16)
                                                .unwrap(),
                                    );
                                    match tmp {
                                        None => self.char_literal = '\u{FFFD}',
                                        Some(val) => self.char_literal = val,
                                    }
                                }
                                self.state = State::EscapeCharacter(Box::new(state), val + 1);
                            }
                            ('}', 3..=8) => {
                                self.state = state;
                            }
                            ('x', 0) => {
                                self.char_literal = '\0';
                                self.state = State::EscapeCharacter(Box::new(state), 11);
                            }
                            ('0'..='9' | 'a'..='f' | 'A'..='F', 11..=12) => {
                                if !self.escape_error {
                                    self.char_literal = (((self.char_literal as u8) << 4)
                                        | u8::from_str_radix(self.c.to_string().as_str(), 16)
                                            .unwrap())
                                        as char;
                                } else {
                                    self.char_literal = '\0';
                                }

                                if val == 12 {
                                    self.state = state;
                                } else {
                                    self.state = State::EscapeCharacter(Box::new(state), val + 1);
                                }
                            }
                            _ => {
                                self.escape_error = true;
                                self.state = state;
                                self.char_literal = '\0';
                                self.stop_reset = true;
                                self.matching = true;

                                self.new_token = Tokenizer::create_token_c(
                                    self.current.index - self.escape_start.index + 1,
                                    self.escape_start.index - 1,
                                    self.current.index_real - self.escape_start.index_real + 1,
                                    self.escape_start.index_real - 1,
                                    self.escape_start.line,
                                    self.escape_start.column - 1,
                                    format!("Invalid character in escape sequence: {}", self.c),
                                );
                            }
                        }
                    }
                    State::CharLiteralNormal() => {
                        match self.c {
                            '\'' => {
                                if self.char_literal == '\0'
                                    || self.char_literal.len_utf8() == self.string.len()
                                {
                                    self.default_reset(
                                        false,
                                        TokenType::CharLiteral(self.char_literal),
                                    )
                                } else {
                                    self.stop_reset = true;
                                    self.new_token = self.create_token(
                                        "Char literal cannot contain more than one character",
                                    );
                                    self.matching = true;
                                    self.char_literal = '\0';
                                    //self.default_reset(false, TokenType::ERROR("Char literal cannot contain more than one character".into()));
                                }
                            }
                            '\\' => {
                                self.state =
                                    State::EscapeCharacter(Box::new(State::CharLiteralEscape()), 0);
                            }
                            '\n' | '\r' => {
                                self.default_reset(true, "Cannot have new line in char literal");
                            }
                            char => {
                                self.char_literal = char;
                                self.string.push(self.char_literal);
                            }
                        }
                    }
                    State::CharLiteralEscape() => {
                        if !self.escape_error {
                            self.string.push(self.c);
                        }
                        self.matching = true;
                        self.state = State::CharLiteralNormal();
                    }
                    State::CharLiteralStart => match self.c {
                        '\'' => self.default_reset(false, "Empty Char Literal"),
                        _ => {
                            self.string = String::new();
                            self.char_literal = '\0';
                            self.state = State::CharLiteralNormal();
                            self.matching = true;
                        }
                    },
                    State::StringStart => match self.c {
                        '"' => {
                            self.new_token = self.create_token(TokenType::StringLiteral("".into()));
                            self.state = State::Default;
                        }
                        _ => {
                            self.string = String::new();
                            self.state = State::StringNormal;
                            self.matching = true;
                        }
                    },
                    State::StringNormal => {
                        match self.c {
                            //'\r'|'\n' => {
                            //self.matching = true;
                            //    self.new_token = self.create_token(TokenType::ERROR("New line in string".into()));
                            //}
                            '\\' => {
                                self.state =
                                    State::EscapeCharacter(Box::new(State::StringEscape), 0);
                            }
                            '"' => {
                                self.new_token = self.create_token(TokenType::StringLiteral(
                                    self.string.to_string(),
                                ));
                                self.state = State::Default;
                            }
                            c => {
                                self.string.push(c);
                            }
                        }
                    }
                    State::StringEscape => {
                        if !self.escape_error {
                            self.string.push(self.char_literal);
                        }
                        self.matching = true;
                        self.state = State::StringNormal;
                    }
                    State::Equal => {
                        match self.c {
                            '=' => self.new_token = self.create_token(TokenType::Equals),
                            _ => {
                                self.matching = true;
                                self.new_token = self.create_token(TokenType::Assignment);
                            }
                        }
                        self.state = State::Default;
                    }
                    State::NumberLiteral(val) => match (self.c, val) {
                        ('e' | 'E', 0) => {
                            self.state = State::NumberLiteral(1);
                        }
                        ('e' | 'E', 1) => {
                            self.state = State::NumberLiteral(0);
                        }
                        ('A'..='Z' | 'a'..='z' | '_' | '0'..='9' | '.', _) => {
                            self.state = State::NumberLiteral(0);
                        }
                        ('+' | '-', 1) => {
                            self.state = State::NumberLiteral(0);
                        }
                        _ => {
                            self.matching = true;
                            self.new_token = self.create_number_token(self.curr_str());
                            self.state = State::Default;
                        }
                    },
                    State::EOF => {}
                }
                match &self.new_token {
                    None => match self.state {
                        State::Default => {}
                        _ => if self.start_curr != self.current {},
                    },
                    Some(_) => {
                        if self.matching {
                            if !self.stop_reset {
                                self.start_curr = self.last;
                            }
                        } else if !self.stop_reset {
                            self.start_curr = self.current;
                        }

                        let mut new = Option::None;
                        mem::swap(&mut self.new_token, &mut new);
                        match new {
                            Option::Some(Result::Ok(Token {
                                t_type: TokenType::Whitespace,
                                ..
                            })) => {
                                if self.include_whitespace {
                                    return new;
                                }
                            }
                            Option::Some(Result::Ok(Token {
                                t_type: TokenType::Comment(_),
                                ..
                            })) => {
                                if self.include_comments {
                                    return new;
                                }
                            }
                            Option::Some(Result::Ok(Token {
                                t_type:
                                    TokenType::OuterDocumentation(_) | TokenType::InnerDocumentation(_),
                                ..
                            })) => {
                                if self.include_documentation {
                                    return new;
                                }
                            }
                            _ => {
                                return new;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a> Tokenizer<'a> {
    #[allow(unused)]
    pub fn from_string(data: &'a String) -> Tokenizer<'a> {
        Tokenizer {
            bytes: data.as_bytes(),
            iterator: data.chars(),
            iterations: 0,
            state: State::Default,

            current: BufferIndex::default(),
            last: BufferIndex::default(),
            start_curr: BufferIndex::default(),
            escape_start: BufferIndex::default(),

            matching: false,
            stop_reset: false,
            new_token: None,
            string: String::new(),
            char_literal: '\0',
            escape_error: false,
            c: '\0',

            ident_mode: IdentifierMode::Unicode,
            include_whitespace: false,
            include_comments: false,
            include_documentation: true,
        }
    }

    #[allow(unused)]
    pub fn from_str(data: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            bytes: data.as_bytes(),
            iterator: data.chars(),
            iterations: 0,
            state: State::Default,

            current: BufferIndex::default(),
            last: BufferIndex::default(),
            start_curr: BufferIndex::default(),
            escape_start: BufferIndex::default(),

            matching: false,
            stop_reset: false,
            new_token: None,
            string: String::new(),
            char_literal: '\0',
            escape_error: false,
            c: '\0',

            ident_mode: IdentifierMode::Unicode,
            include_whitespace: false,
            include_comments: false,
            include_documentation: true,
        }
    }

    fn is_curr_ident_start(&self) -> bool {
        self.is_ident_start(self.c)
    }
    fn is_curr_ident_continue(&self) -> bool {
        self.is_ident_continue(self.c)
    }
    fn is_ident_start(&self, c: char) -> bool {
        return match self.ident_mode {
            IdentifierMode::Ascii => match c {
                'A'..='Z' | 'a'..='z' | '_' | '.' | '?' => true,
                _ => false,
            },
            IdentifierMode::Unicode => {
                unicode_xid::UnicodeXID::is_xid_start(c) || c == '_' || c == '.' || c == '?'
            }
        };
    }
    fn is_ident_continue(&self, c: char) -> bool {
        return match self.ident_mode {
            IdentifierMode::Ascii => match c {
                '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '$' | '#' | '@' | '~' | '.' | '?' => true,
                _ => false,
            },
            IdentifierMode::Unicode => {
                unicode_xid::UnicodeXID::is_xid_continue(c)
                    || match c {
                        '_' | '$' | '#' | '@' | '~' | '.' | '?' => true,
                        _ => false,
                    }
            }
        };
    }

    #[allow(unused)]
    pub fn reset(&mut self) {
        self.iterations = 0;
        self.state = State::Default;
        self.iterator = util::tokenizer::chars_from_u8(self.bytes);

        self.current = BufferIndex::default();
        self.last = BufferIndex::default();
        self.start_curr = BufferIndex::default();
        self.escape_start = BufferIndex::default();

        self.matching = false;
        self.stop_reset = false;
        self.new_token = None;
        self.string = String::new();
        self.char_literal = '\0';
        self.escape_error = false;
        self.c = '\0';
    }

    fn create_token(&self, t_type: impl IntoWithData<TokenizerItem>) -> TokenizerItem {
        let mut data = TokenData {
            index: self.start_curr.index,
            size: (self.current.index - self.start_curr.index),
            size_real: (self.current.index_real - self.start_curr.index_real),
            index_real: self.start_curr.index_real,
            line: self.start_curr.line,
            column: self.start_curr.column,
            file: None,
        };
        if self.matching {
            data.size -= 1;
            data.size_real -= self.c.len_utf8();
        };
        t_type.into_w_data(data)
    }

    // fn create_token(&self, t_type: TokenType) -> TokenizerItem {
    //     let mut temp = Token{t_type,
    //         t_data: TokenData{
    //             index: self.start_curr.index,
    //             size: (self.current.index - self.start_curr.index),
    //             size_real: (self.current.index_real - self.start_curr.index_real),
    //             index_real: self.start_curr.index_real,
    //             line: self.start_curr.line,
    //             column: self.start_curr.column
    //         }
    //     };
    //     if self.matching {
    //         temp.t_data.size -= 1;
    //         temp.t_data.size_real -= self.c.len_utf8();
    //     }
    //     Option::Some(Result::Ok(temp))
    // }

    fn create_token_c(
        size: usize,
        index: usize,
        size_real: usize,
        index_real: usize,
        line: usize,
        column: usize,
        t_type: impl IntoWithData<TokenizerItem>,
    ) -> TokenizerItem {
        let data = TokenData {
            size,
            index,
            size_real,
            index_real,
            line,
            column,
            file: None,
        };
        t_type.into_w_data(data)
    }

    fn create_number_token(&self, mut num: String) -> TokenizerItem {
        let original: String = num.to_string();
        let suffixes = [
            "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64",
        ];
        let prefixes = ["0x", "0b"];

        let mut suffix: &str = "";
        let mut prefix = "";
        for suf in suffixes {
            if num.ends_with(suf) {
                suffix = suf;
                num = num.as_str()[0..num.len() - suf.len()].to_string();
                break;
            }
        }
        for pre in prefixes {
            if num.starts_with(pre) {
                prefix = pre;
                num = num.replacen(pre, "", 1);
                break;
            }
        }

        num = num.replace("_", "");

        let base;
        match prefix {
            "0x" => base = 16,
            "0b" => base = 2,
            _ => base = 10,
        }

        let t_type: Result<TokenType, String>;

        macro_rules! parse_to_token {
            ($a: path, $b: ident) => {
                match num.parse::<$a>() {
                    Ok(val) => t_type = Result::Ok(TokenType::$b(val)),
                    Err(val) => t_type = Result::Err(val.to_string()),
                }
            };
        }

        macro_rules! parse_to_token_base {
            ($a: ident, $b: ident, $c: ident) => {
                match $a::from_str_radix(num.as_str(), $c) {
                    Ok(val) => t_type = Result::Ok(TokenType::$b(val)),
                    Err(val) => t_type = Result::Err(val.to_string()),
                }
            };
        }

        match (suffix, base) {
            ("i8", _) => parse_to_token_base!(i8, I8Literal, base),
            ("i16", _) => parse_to_token_base!(i16, I16Literal, base),
            ("i32", _) => parse_to_token_base!(i32, I32Literal, base),
            ("i64", _) => parse_to_token_base!(i64, I64Literal, base),
            ("i128", _) => parse_to_token_base!(i128, I128Literal, base),
            ("u8", _) => parse_to_token_base!(u8, U8Literal, base),
            ("u16", _) => parse_to_token_base!(u16, U16Literal, base),
            ("u32", _) => parse_to_token_base!(u32, U32Literal, base),
            ("u64", _) => parse_to_token_base!(u64, U64Literal, base),
            ("u128", _) => parse_to_token_base!(u128, U128Literal, base),
            ("f32", 10) => parse_to_token!(f32, F32Literal),
            ("f64", 10) => parse_to_token!(f64, F64Literal),
            ("f32" | "f64", _) => {
                t_type = Result::Err("Cannot have non base 10 floating point literal".into())
            }
            (_, base) => {
                if base == 10 {
                    if num.contains("e") || num.contains("E") || num.contains(".") {
                        parse_to_token!(f32, F32Literal)
                    } else {
                        parse_to_token_base!(i32, I32Literal, base)
                    }
                } else {
                    parse_to_token_base!(i32, I32Literal, base)
                }
            }
        }
        match t_type {
            Ok(ok) => self.create_token(ok),
            Err(err) => self.create_token(format!("{} for: {}", err, original)),
        }
    }

    fn create_identifier_or_keyword(&self, ident: String) -> TokenizerItem {
        let t_type: TokenType;
        match ident.as_str() {
            "true" => t_type = TokenType::BoolLiteral(true),
            "false" => t_type = TokenType::BoolLiteral(false),
            "0" => t_type = TokenType::Register(0),
            "1" => t_type = TokenType::Register(1),
            "2" => t_type = TokenType::Register(2),
            "3" => t_type = TokenType::Register(3),
            "4" => t_type = TokenType::Register(4),
            "5" => t_type = TokenType::Register(5),
            "6" => t_type = TokenType::Register(6),
            "7" => t_type = TokenType::Register(7),
            "8" => t_type = TokenType::Register(8),
            "9" => t_type = TokenType::Register(9),
            "10" => t_type = TokenType::Register(10),
            "11" => t_type = TokenType::Register(11),
            "12" => t_type = TokenType::Register(12),
            "13" => t_type = TokenType::Register(13),
            "14" => t_type = TokenType::Register(14),
            "15" => t_type = TokenType::Register(15),
            "16" => t_type = TokenType::Register(16),
            "17" => t_type = TokenType::Register(17),
            "18" => t_type = TokenType::Register(18),
            "19" => t_type = TokenType::Register(19),
            "20" => t_type = TokenType::Register(20),
            "21" => t_type = TokenType::Register(21),
            "22" => t_type = TokenType::Register(22),
            "23" => t_type = TokenType::Register(23),
            "24" => t_type = TokenType::Register(24),
            "25" => t_type = TokenType::Register(25),
            "26" => t_type = TokenType::Register(26),
            "27" => t_type = TokenType::Register(27),
            "28" => t_type = TokenType::Register(28),
            "29" => t_type = TokenType::Register(29),
            "30" => t_type = TokenType::Register(30),
            "31" => t_type = TokenType::Register(31),

            _ => t_type = TokenType::Identifier(ident),
        }

        self.create_token(t_type)
    }

    fn default_reset(&mut self, matching: bool, tok: impl IntoWithData<TokenizerItem>) {
        self.matching = matching;
        self.new_token = self.create_token(tok);
        self.state = State::Default;
    }

    #[allow(unused)]
    pub fn string_from_token(&self, token: &Token) -> String {
        String::from_utf8_lossy(
            &self.bytes[token.get_real_index()..token.get_real_index() + token.get_real_size()],
        )
        .to_string()
    }
    #[allow(unused)]
    pub fn str_from_token(&self, token: &Token) -> &str {
        std::str::from_utf8(
            &self.bytes[token.get_real_index()..token.get_real_index() + token.get_real_size()],
        )
        .expect("")
    }
    #[allow(unused)]
    pub fn str_from_token_data(&self, t_data: &TokenData) -> &str {
        std::str::from_utf8(
            &self.bytes[t_data.get_real_index()..t_data.get_real_index() + t_data.get_real_size()],
        )
        .expect("")
    }

    pub fn curr_str(&self) -> String {
        if self.matching {
            String::from_utf8_lossy(
                &self.bytes[self.start_curr.index_real as usize
                    ..(self.current.index_real - self.c.len_utf8()) as usize],
            )
            .to_string()
        } else {
            String::from_utf8_lossy(
                &self.bytes[self.start_curr.index_real as usize..self.current.index_real as usize],
            )
            .to_string()
        }
    }
    fn ntm(&mut self) {
        self.last = self.current;

        self.current.index += 1;
        self.current.index_real += self.c.len_utf8();
        self.current.column += 1;
        self.stop_reset = false;
    }

    #[allow(unused)]
    pub fn ident_mode(mut self, ident_mode: IdentifierMode) -> Self {
        self.ident_mode = ident_mode;
        self
    }
    #[allow(unused)]
    pub fn include_whitespace(mut self, include_whitespace: bool) -> Self {
        self.include_whitespace = include_whitespace;
        self
    }
    #[allow(unused)]
    pub fn include_comments(mut self, include_comments: bool) -> Self {
        self.include_comments = include_comments;
        self
    }
    #[allow(unused)]
    pub fn include_documentation(mut self, include_documentation: bool) -> Self {
        self.include_documentation = include_documentation;
        self
    }

    pub fn tokenize(&mut self) -> Vec<Result<Token, TokenizerError>> {
        let mut tokens = Vec::new();
        loop {
            match self.next() {
                None => {
                    return tokens;
                }
                Some(tok) => tokens.push(tok),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_tokenizer() {}
}
