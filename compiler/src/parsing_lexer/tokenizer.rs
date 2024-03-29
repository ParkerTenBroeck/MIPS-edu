use std::mem;
use std::str::Chars;

pub type Token = util::token::Token<TokenType>;
pub type TokenData = util::token::TokenData;
pub type BufferIndex = util::tokenizer::BufferIndex;

//test
#[derive(Debug, Clone)]
pub enum TokenType {
    VoidKeyword,
    StructKeyword,
    AsmKeyword,
    ConstKeyword,
    StaticKeyword,
    SizeofKeyword,
    EnumKeyword,
    FnKeyword,
    PubKeyword,
    SuperKeyword,
    SelfKeyword,
    LetKeyword,

    IfKeyword,
    ElseKeyword,
    WhileKeyword,
    DoKeyword,
    ReturnKeyword,
    ForKeyword,
    BreakKeyword,
    SwitchKeyword,
    CaseKeyword,
    GotoKeyword,
    RestrictKeyword,

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
    Arrow,
    Comma,
    Colon,
    ColonColon,
    Semicolon,
    QuestionMark,
    At,
    Octothorp,
    Dollar,
    DotDotDot,
    Increment,
    Decrement,

    LessThan,
    LessThanEq,
    GreaterThan,
    GreaterThanEq,
    Equals,
    NotEquals,

    AssignmentAdd,
    AssignmentSub,
    AssignmentMul,
    AssignmentDiv,
    AssignmentMod,
    AssignmentAnd,
    AssignmentOr,
    AssignmentXor,
    AssignmentShiftLeft,
    AssignmentShiftRight,
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

    Whitespace,

    Comment(String),
    OuterDocumentation(String),
    InnerDocumentation(String),

    Error(String),
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
    new_token: Option<Token>,

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
    //CarriageReturn,
    NumberLiteral(i32),

    Identifier,
    R,
    RawIdentStart,
    RawIdentContinue,

    StringStart,
    StringNormal,
    StringEscape,
    CharLiteralStart,
    CharLiteralNormal(),
    CharLiteralEscape(),

    LineComment(bool),
    BlockComment(u8, u32),

    OuterDoc(bool),
    OuterBlockDoc(u8, u32),
    InnerDoc,
    InnerBlockDoc(u8, u32),

    Colon,
    Equal,
    Dot(i32),
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

    Eof,
}

impl Iterator for Tokenizer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (&self.state, self.c) {
                (State::Eof, '\0') => {
                    return Option::None;
                }
                (State::Default, '\0') => {}
                (_, '\0') => {
                    self.matching = true;
                    let tmp = self.create_token(TokenType::Error(format!(
                        "Reached EOF while in state: {:?}",
                        self.state
                    )));
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
                    State::Default => match self.c {
                        '\0' => {
                            self.state = State::Eof;
                        }

                        ' ' | '\t' | '\r' | '\n' => {
                            self.state = State::Whitespace;
                        }

                        '"' => self.state = State::StringStart,
                        '\'' => self.state = State::CharLiteralStart,
                        '.' => self.state = State::Dot(0),
                        '<' => self.state = State::LessThan,
                        '>' => self.state = State::GreaterThan,
                        '!' => self.state = State::Bang,
                        '&' => self.state = State::And,
                        '|' => self.state = State::Or,
                        '^' => self.state = State::Xor,
                        '/' => self.state = State::Div,
                        '%' => self.state = State::Mod,
                        '-' => self.state = State::Minus,
                        '+' => self.state = State::Plus,
                        '*' => self.state = State::Star,
                        '=' => self.state = State::Equal,
                        ':' => self.state = State::Colon,

                        '(' => self.new_token = self.create_token(TokenType::LPar),
                        ')' => self.new_token = self.create_token(TokenType::RPar),
                        '{' => self.new_token = self.create_token(TokenType::LBrace),
                        '}' => self.new_token = self.create_token(TokenType::RBrace),
                        '[' => self.new_token = self.create_token(TokenType::LBracket),
                        ']' => self.new_token = self.create_token(TokenType::RBracket),
                        ';' => self.new_token = self.create_token(TokenType::Semicolon),
                        ',' => self.new_token = self.create_token(TokenType::Comma),
                        '~' => self.new_token = self.create_token(TokenType::BitwiseNot),
                        '?' => self.new_token = self.create_token(TokenType::QuestionMark),
                        '@' => self.new_token = self.create_token(TokenType::At),
                        '#' => self.new_token = self.create_token(TokenType::Octothorp),
                        '$' => self.new_token = self.create_token(TokenType::Dollar),

                        'r' => self.state = State::R,
                        '0'..='9' => self.state = State::NumberLiteral(0),

                        _ => {
                            if self.is_curr_ident_start() {
                                self.state = State::Identifier;
                            } else if self.c.is_whitespace() {
                                self.state = State::Whitespace;
                            } else {
                                let message = format!("Unexpected Char: {}", self.c);
                                self.new_token = self.create_token(TokenType::Error(message));
                            }
                        }
                    },
                    State::Whitespace => match self.c {
                        ' ' | '\t' | '\r' | '\n' => {
                            self.state = State::Whitespace;
                        }
                        _ => {
                            if self.c.is_whitespace() {
                                self.state = State::Whitespace;
                            } else {
                                self.default_reset(true, TokenType::Whitespace);
                            }
                        }
                    },
                    /***
                     */
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
                    State::RawIdentStart => {
                        if self.is_curr_ident_start() {
                            self.state = State::RawIdentContinue;
                        } else {
                            self.default_reset(
                                true,
                                TokenType::Error(
                                    "Raw identifier must be one or more characters".into(),
                                ),
                            );
                        }
                    }
                    State::RawIdentContinue => {
                        if self.is_curr_ident_continue() {
                            self.state = State::RawIdentContinue;
                        } else {
                            self.matching = true;
                            let ident = self.curr_str().replacen("r#", "", 1);
                            self.new_token = self.create_token(TokenType::Identifier(ident));
                            self.state = State::Default;
                        }
                    }
                    State::R => match self.c {
                        '#' => {
                            self.state = State::RawIdentStart;
                            self.matching = false;
                        }
                        _ => {
                            if self.is_ident_start('r') {
                                self.state = State::Identifier;
                            } else {
                                self.default_reset(
                                    true,
                                    TokenType::Error("unknown character r".into()),
                                );
                            }
                            self.matching = true;
                        }
                    },
                    State::Colon => match self.c {
                        ':' => self.default_reset(false, TokenType::ColonColon),
                        _ => self.default_reset(true, TokenType::Colon),
                    },
                    State::ShiftLeft => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentShiftLeft),
                        _ => self.default_reset(true, TokenType::ShiftLeft),
                    },
                    State::ShiftRight => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentShiftRight),
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
                        '+' => self.default_reset(false, TokenType::Increment),
                        '=' => self.default_reset(false, TokenType::AssignmentAdd),
                        _ => self.default_reset(true, TokenType::Plus),
                    },
                    State::Minus => match self.c {
                        '-' => self.default_reset(false, TokenType::Decrement),
                        '=' => self.default_reset(false, TokenType::AssignmentSub),
                        '>' => self.default_reset(false, TokenType::Arrow),
                        _ => self.default_reset(true, TokenType::Minus),
                    },
                    State::Star => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentMul),
                        _ => self.default_reset(true, TokenType::Star),
                    },
                    State::Div => match self.c {
                        '/' => self.state = State::LineComment(false),
                        '*' => self.state = State::BlockComment(0, 0),
                        '=' => self.default_reset(false, TokenType::AssignmentDiv),
                        _ => self.default_reset(true, TokenType::Slash),
                    },
                    State::Mod => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentMod),
                        _ => self.default_reset(true, TokenType::Percent),
                    },
                    State::Xor => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentXor),
                        _ => self.default_reset(true, TokenType::BitwiseXor),
                    },
                    State::Or => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentOr),
                        '|' => self.default_reset(false, TokenType::LogicalOr),
                        _ => self.default_reset(true, TokenType::BitwiseOr),
                    },
                    State::And => match self.c {
                        '=' => self.default_reset(false, TokenType::AssignmentAnd),
                        '&' => self.default_reset(false, TokenType::LogicalAnd),
                        _ => self.default_reset(true, TokenType::Ampersand),
                    },
                    State::Dot(val) => match (self.c, val) {
                        ('.', 0) => {
                            self.state = State::Dot(val + 1);
                        }
                        ('.', 1) => {
                            self.new_token = self.create_token(TokenType::DotDotDot);
                            self.state = State::Default;
                        }
                        (_, 0) => {
                            self.matching = true;
                            self.new_token = self.create_token(TokenType::Dot);
                            self.state = State::Default;
                        }
                        _ => {
                            self.matching = true;
                            self.new_token = self.create_token(TokenType::Error(format!(
                                "Incorrect number of dots: {}",
                                val + 1
                            )));
                            self.state = State::Default;
                        }
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
                                    TokenType::Error(format!(
                                        "Invalid character in escape sequence: {}",
                                        self.c
                                    )),
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
                                    self.new_token = self.create_token(TokenType::Error(
                                        "Char literal cannot contain more than one character"
                                            .into(),
                                    ));
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
                                self.default_reset(
                                    true,
                                    TokenType::Error("Cannot have new line in char literal".into()),
                                );
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
                        '\'' => {
                            self.default_reset(false, TokenType::Error("Empty Char Literal".into()))
                        }
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
                    State::Identifier => {
                        if self.is_curr_ident_continue() {
                            self.state = State::Identifier;
                        } else {
                            self.matching = true;
                            let ident = self.curr_str();
                            self.new_token = self.create_identifier_or_keyword(ident); //self.create_token(TokenType::Identifier(ident.to_string()));
                            self.state = State::Default;
                        }
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
                    State::Eof => {}
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
                            Option::Some(Token {
                                t_type: TokenType::Whitespace,
                                ..
                            }) => {
                                if self.include_whitespace {
                                    return new;
                                }
                            }
                            Option::Some(Token {
                                t_type: TokenType::Comment(_),
                                ..
                            }) => {
                                if self.include_comments {
                                    return new;
                                }
                            }
                            Option::Some(Token {
                                t_type:
                                    TokenType::OuterDocumentation(_) | TokenType::InnerDocumentation(_),
                                ..
                            }) => {
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
    pub fn new_from_string(data: &'a String) -> Tokenizer<'a> {
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
    pub fn new_from_str(data: &'a str) -> Tokenizer<'a> {
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
        matches!(c, 'A'..='Z' | 'a'..='z' | '_')
    }
    fn is_ident_continue(&self, c: char) -> bool {
        matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_')
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

    fn create_token(&self, t_type: TokenType) -> Option<Token> {
        let mut temp = Token {
            t_type,
            t_data: TokenData {
                index: self.start_curr.index,
                size: (self.current.index - self.start_curr.index),
                size_real: (self.current.index_real - self.start_curr.index_real),
                index_real: self.start_curr.index_real,
                line: self.start_curr.line,
                column: self.start_curr.column,
                file: None,
            },
        };
        if self.matching {
            temp.t_data.size -= 1;
            temp.t_data.size_real -= self.c.len_utf8();
        }
        Option::Some(temp)
    }

    fn create_token_c(
        size: usize,
        index: usize,
        size_real: usize,
        index_real: usize,
        line: usize,
        column: usize,
        t_type: TokenType,
    ) -> Option<Token> {
        Option::Some(Token {
            t_type,
            t_data: TokenData {
                size,
                index,
                size_real,
                index_real,
                line,
                column,
                file: None,
            },
        })
    }

    fn create_number_token(&self, mut num: String) -> Option<Token> {
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

        num = num.replace('_', "");

        let base = match prefix {
            "0x" => 16,
            "0b" => 2,
            _ => 10,
        };

        let mut t_type: TokenType;

        macro_rules! parse_to_token {
            ($a: path, $b: ident) => {
                match num.parse::<$a>() {
                    Ok(val) => t_type = TokenType::$b(val),
                    Err(val) => t_type = TokenType::Error(val.to_string()),
                }
            };
        }

        macro_rules! parse_to_token_base {
            ($a: ident, $b: ident, $c: ident) => {
                match $a::from_str_radix(num.as_str(), $c) {
                    Ok(val) => t_type = TokenType::$b(val),
                    Err(val) => t_type = TokenType::Error(val.to_string()),
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
                t_type = TokenType::Error("Cannot have non base 10 floating point literal".into())
            }
            (_, base) => {
                if base == 10 {
                    if num.contains('e') || num.contains('E') || num.contains('.') {
                        parse_to_token!(f32, F32Literal)
                    } else {
                        parse_to_token_base!(i32, I32Literal, base)
                    }
                } else {
                    parse_to_token_base!(i32, I32Literal, base)
                }
            }
        }
        if let TokenType::Error(string) = t_type {
            t_type = TokenType::Error(format!("{} for: {}", string, original))
        }

        self.create_token(t_type)
    }

    fn create_identifier_or_keyword(&self, ident: String) -> Option<Token> {
        let t_type = match ident.as_str() {
            "void" => TokenType::VoidKeyword,
            "struct" => TokenType::StructKeyword,
            "asm" => TokenType::AsmKeyword,
            "const" => TokenType::ConstKeyword,
            "static" => TokenType::StaticKeyword,
            "sizeof" => TokenType::SizeofKeyword,
            "enum" => TokenType::EnumKeyword,
            "fn" => TokenType::FnKeyword,
            "pub" => TokenType::PubKeyword,
            "self" => TokenType::SelfKeyword,
            "super" => TokenType::SuperKeyword,
            "let" => TokenType::LetKeyword,
            "if" => TokenType::IfKeyword,
            "else" => TokenType::ElseKeyword,
            "while" => TokenType::WhileKeyword,
            "do" => TokenType::DoKeyword,
            "for" => TokenType::ForKeyword,
            "return" => TokenType::ReturnKeyword,
            "break" => TokenType::BreakKeyword,
            "switch" => TokenType::SwitchKeyword,
            "case" => TokenType::CaseKeyword,
            "goto" => TokenType::GotoKeyword,
            "restrict" => TokenType::RestrictKeyword,
            "true" => TokenType::BoolLiteral(true),
            "false" => TokenType::BoolLiteral(false),
            _ => TokenType::Identifier(ident),
        };

        self.create_token(t_type)
    }

    fn default_reset(&mut self, matching: bool, tok: TokenType) {
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
                &self.bytes
                    [self.start_curr.index_real..(self.current.index_real - self.c.len_utf8())],
            )
            .to_string()
        } else {
            String::from_utf8_lossy(
                &self.bytes[self.start_curr.index_real..self.current.index_real],
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

    pub fn tokenize(&mut self) -> Vec<Token> {
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
