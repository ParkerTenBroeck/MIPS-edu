use std::fmt::{Display, Formatter};
use std::mem;
use std::str::Chars;


#[derive(Debug)]
pub enum TokenType{
    VoidKeyword,
    StructKeyword,
    AsmKeyword,
    ConstKeyword,
    StaticKeyword,
    SizeofKeyword,
    EnumKeyword,

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


    ISizeKeyword,
    USizeKeyword,
    I8Keyword,
    I16Keyword,
    I32Keyword,
    U8Keyword,
    U16Keyword,
    U32Keyword,
    CharKeyword,
    BoolKeyword,

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
    Semicolon,
    QuestionMark,
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
    AssignmentAnd,
    AssignmentOr,
    AssignmentXor,
    AssignmentShiftLeft,
    AssignmentShiftRight,
    AssignmentMod,
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

    ERROR(String)
}

#[derive(Debug)]
pub struct Token{
    pub(in crate::parsing_lexer)  t_type: TokenType,
    size: usize,
    index: usize,
    index_real:usize,
    size_real:usize,
    line: usize,
    column: usize
}

//impl Token{
//    pub fn get_token_type(&self) -> &TokenType{
//        &self.t_type
//    }
//}
impl Display for Token{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        if self.index < -1isize as usize {
            write!(f, "line:{}, size:{}, column:{} type: {:?}",self.line + 1, self.size,self.column, self.t_type)
        }else{
            write!(f, "1")
        }
    }
}

impl Iterator for Tokenizer<'_>{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {

        loop{

            match (&self.state, self.c){
                (State::EOF, '\0') => {
                    return Option::None;
                }
                (State::Default, '\0') =>{}
                (_, '\0') => {
                    let tmp = self.create_token(TokenType::ERROR(format!("Invalid State at EOF: {:?}", self.state)));
                    self.state = State::Default;
                    return tmp;
                }
                _ => {}
            }

            if !self.matching {
                match self.iterator.next(){
                    Some(char) => {
                        self.c = char;
                        self.iterations += 1;
                        self.ntm();
                    }
                    None => {
                        match self.state{
                            State::Default => {
                                return Option::None;
                            }
                            _ => {
                                self.c = '\0';
                                self.iterations +=1;
                                self.ntm();
                            }
                        }

                    }
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
                            ' ' | '\t' => {
                                self.start_curr = self.current;
                            }
                            '\r' => {
                                self.state = State::CarriageReturn;
                            }
                            '\n' => {
                                self.current.column = 0;
                                self.current.line += 1;
                                self.start_curr = self.current;
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

                            '(' => self.new_token = self.create_token(TokenType::LPar),
                            ')' => self.new_token = self.create_token(TokenType::RPar),
                            '{' => self.new_token = self.create_token(TokenType::LBrace),
                            '}' => self.new_token = self.create_token(TokenType::RBrace),
                            '[' => self.new_token = self.create_token(TokenType::LBracket),
                            ']' => self.new_token = self.create_token(TokenType::RBracket),
                            ';' => self.new_token = self.create_token(TokenType::Semicolon),
                            ':' => self.new_token = self.create_token(TokenType::Colon),
                            ',' => self.new_token = self.create_token(TokenType::Comma),
                            '~' => self.new_token = self.create_token(TokenType::BitwiseNot),
                            '?' => self.new_token = self.create_token(TokenType::QuestionMark),

                            'A'..='Z'|'a'..='z' => {self.state =State::Identifier ;}
                            '0'..='9' => {self.state = State::NumberLiteral(0);}

                            _ => {
                                let message = format!("Unexpected Char: {}", self.c);
                                self.new_token = self.create_token(TokenType::ERROR(message));
                            }
                        }
                    }
                    State::ShiftLeft => {
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentShiftLeft),
                            _ => self.default_reset(true, TokenType::ShiftLeft),
                        }
                    }
                    State::ShiftRight => {
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentShiftRight),
                            _ => self.default_reset(true, TokenType::ShiftRight),
                        }
                    }
                    State::GreaterThan =>{
                        match self.c {
                            '>' => self.state = State::ShiftRight,
                            '=' => self.default_reset(false, TokenType::GreaterThanEq),
                            _ => self.default_reset(true, TokenType::GreaterThan),
                        }
                    }
                    State::LessThan =>{
                        match self.c {
                            '<' => self.state = State::ShiftLeft,
                            '=' => self.default_reset(false, TokenType::LessThanEq),
                            _ => self.default_reset(true, TokenType::LessThan),
                        }
                    }
                    State::Bang =>{
                        match self.c {
                            '=' => self.default_reset(false, TokenType::NotEquals),
                            _ => self.default_reset(true, TokenType::LogicalNot),
                        }
                    }
                    State::Plus =>{
                        match self.c {
                            '+' => self.default_reset(false, TokenType::Increment),
                            '=' => self.default_reset(false, TokenType::AssignmentAdd),
                            _ => self.default_reset(true, TokenType::Plus),
                        }
                    }
                    State::Minus =>{
                        match self.c {
                            '-' => self.default_reset(false, TokenType::Decrement),
                            '=' => self.default_reset(false, TokenType::AssignmentSub),
                            '>' => self.default_reset(false, TokenType::Arrow),
                            _ => self.default_reset(true, TokenType::Minus),
                        }
                    }
                    State::Star =>{
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentMul),
                            _ => self.default_reset(true, TokenType::Star),
                        }
                    }
                    State::Div =>{
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentDiv),
                            _ => self.default_reset(true, TokenType::Slash),
                        }
                    }
                    State::Mod =>{
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentMod),
                            _ => self.default_reset(true, TokenType::Percent),
                        }
                    }
                    State::Xor =>{
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentXor),
                            _ => self.default_reset(true, TokenType::BitwiseXor),
                        }
                    }
                    State::Or =>{
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentOr),
                            '|' => self.default_reset(false, TokenType::LogicalOr),
                            _ => self.default_reset(true, TokenType::BitwiseOr),
                        }
                    }
                    State::And => {
                        match self.c {
                            '=' => self.default_reset(false, TokenType::AssignmentAnd),
                            '&' => self.default_reset(false, TokenType::LogicalAnd),
                            _ => self.default_reset(true, TokenType::Ampersand),
                        }
                    }
                    State::Dot(val) =>{
                        match (self.c, val){
                            ('.',0) =>{
                                self.state = State::Dot(val + 1);
                            }
                            ('.', 1) =>{
                                self.new_token = self.create_token(TokenType::DotDotDot);
                                self.state = State::Default;
                            }
                            (_,0) => {
                                self.matching = true;
                                self.new_token = self.create_token(TokenType::Dot);
                                self.state = State::Default;
                            }
                            _ =>{
                                self.matching = true;
                                self.new_token = self.create_token(TokenType::ERROR(format!("Incorrect number of dots: {}", val + 1).into()));
                                self.state = State::Default;
                            }
                        }
                    }
                    State::EscapeCharacter(_,_) =>{
                        let mut t = State::Default;
                        mem::swap(&mut self.state, &mut t);
                        let state;
                        let val;
                        match t {
                            State::EscapeCharacter(i_state,i_val) => {
                                state = *i_state;
                                val = i_val;
                            }
                            _ => {panic!()}
                        }
                        if val == 0 {
                            //self.escape_start.index = self.last.index;
                            //self.escape_start.index_real = self.last.index_real;
                            //self.escape_start.line = self.last.line;
                            //self.escape_start.column = self.last.column;
                            self.escape_start = self.last;
                            self.escape_error = false;
                        }
                        match (self.c, val){
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
                            ('{', 1) =>{
                                self.state = State::EscapeCharacter(Box::new(state), 2);
                            }
                            ('0'..='9'|'a'..='f'|'A'..='F', 2..=7) =>{
                                if !self.escape_error{
                                    let tmp = char::from_u32(
                                        ((self.char_literal as u32 ) << 4)
                                            | u32::from_str_radix(self.c.to_string().as_str(), 16).unwrap());
                                    match tmp{
                                        None => {self.char_literal = '\u{FFFD}'}
                                        Some(val) => {self.char_literal = val}
                                    }
                                }
                                self.state = State::EscapeCharacter(Box::new(state), val + 1);
                            }
                            ('}', 3..=8) =>{
                                self.state = state;
                            }
                            ('x', 0) => {
                                self.char_literal = '\0';
                                self.state = State::EscapeCharacter(Box::new(state), 11);
                            }
                            ('0'..='9'|'a'..='f'|'A'..='F', 11..=12) => {
                                if !self.escape_error {
                                    self.char_literal = (((self.char_literal as u8 ) << 4)
                                        | u8::from_str_radix(self.c.to_string().as_str(), 16).unwrap()) as char;
                                }else{
                                    self.char_literal = '\0';
                                }

                                if val == 12{
                                    self.state = state;
                                }else{
                                    self.state = State::EscapeCharacter(Box::new(state), val + 1);
                                }
                            }
                            _ => {
                                self.escape_error = true;
                                self.state = state;
                                self.char_literal = '\0';
                                self.stop_reset = true;
                                self.matching = true;

                                self.new_token = Tokenizer::create_token_c(self.current.index - self.escape_start.index + 2, self.escape_start.index,
                                                                           self.current.index_real - self.escape_start.index_real + 2, self.escape_start.index_real,
                                                                           self.escape_start.line, self.escape_start.column - 1,
                                                                           TokenType::ERROR(format!("Invalid character in escape sequence: {}", self.c)));
                            }
                        }
                    }
                    State::CharLiteralNormal() =>{
                        match self.c {
                            '\'' =>{
                                if self.char_literal.len_utf8() == self.string.len() {
                                    self.default_reset(false, TokenType::CharLiteral(self.char_literal))
                                }else{
                                    self.default_reset(false, TokenType::ERROR("Char literal cannot contain more than one character".into()));
                                }
                            }
                            '\\'=>{
                                self.state = State::EscapeCharacter(Box::new(State::CharLiteralEscape()), 0);
                            }
                            '\n'|'\r' => {
                                self.default_reset(
                                    true, TokenType::ERROR("Cannot have new line in char literal".into()));
                            }
                            char =>{
                                self.char_literal = char;
                                self.string.push(self.char_literal);
                            }
                        }
                    }
                    State::CharLiteralEscape() =>{
                        if !self.escape_error{
                            self.string.push(self.c);
                        }
                        self.matching = true;
                        self.state = State::CharLiteralNormal();
                    }
                    State::CharLiteralStart => {
                        match self.c {
                            '\'' => self.default_reset(false, TokenType::ERROR("Empty Char Literal".into())),
                            _ =>{
                                self.string = String::new();
                                self.char_literal = '\0';
                                self.state = State::CharLiteralNormal();
                                self.matching = true;
                            }
                        }
                    }
                    State::StringStart =>{
                        match self.c {
                            '"' => {
                                self.new_token = self.create_token(TokenType::StringLiteral("".into()));
                                self.state = State::Default;
                            }
                            _ =>{
                                self.string = String::new();
                                self.state = State::StringNormal;
                                self.matching = true;
                            }
                        }
                    }
                    State::StringNormal =>{
                        match self.c {
                            '\r'|'\n' => {
                                self.matching = true;
                                self.new_token = self.create_token(TokenType::ERROR("New line in string".into()));
                            }
                            '\\' => {
                                self.state =  State::EscapeCharacter(Box::new(State::StringEscape), 0);
                            }
                            '"' => {
                                self.new_token = self.create_token(TokenType::StringLiteral(self.string.to_string()));
                                self.state = State::Default;
                            }
                            c =>{
                                self.string.push(c);
                            }
                        }
                    }
                    State::StringEscape => {
                        if !self.escape_error{
                            self.string.push(self.char_literal);
                        }
                        self.matching = true;
                        self.state = State::StringNormal;
                    }
                    State::Equal => {
                        match self.c {
                            '=' =>{
                                self.new_token = self.create_token(TokenType::Equals)
                            }
                            _ =>{
                                self.matching = true;
                                self.new_token = self.create_token(TokenType::Assignment);
                            }
                        }
                        self.state = State::Default;
                    }
                    State::Identifier =>{
                        match self.c {
                            'A'..='Z'|'a'..='z'|'_'|'0'..='9' => {
                                self.state = State::Identifier;
                            }
                            _ =>{
                                self.matching = true;
                                let ident = self.curr_str();
                                self.new_token = self.create_identifier_or_keyword(ident);//self.create_token(TokenType::Identifier(ident.to_string()));
                                self.state = State::Default;
                            }
                        }
                    }
                    State::NumberLiteral(val) => {
                        match (self.c, val){
                            ('e'|'E',0) =>{
                                self.state = State::NumberLiteral(1);
                            }
                            ('e'|'E',1) =>{
                                self.state = State::NumberLiteral(0);
                            }
                            ('A'..='Z'|'a'..='z'|'_'|'0'..='9'|'.',_) => {
                                self.state = State::NumberLiteral(0);
                            }
                            ('+'|'-', 1) =>{
                                self.state =State::NumberLiteral(0);
                            }
                            _ => {
                                self.matching = true;
                                self.new_token = self.create_number_token(self.curr_str());
                                self.state = State::Default;
                            }
                        }
                    }
                    State::CarriageReturn => {
                        match self.c {
                            '\n' => {
                                self.current.line += 1;
                                self.current.column = 0;
                                self.start_curr = self.current;
                                self.state = State::Default;
                            }
                            _ => {
                                self.matching = true;
                                self.new_token = self.create_token(TokenType::ERROR(String::from("Strange carriage return")));
                                self.state = State::Default;
                            }
                        }
                    }
                    State::EOF => {

                    }
                }
                match &self.new_token{
                    None => {}
                    Some(_) => {

                        if self.matching {
                            if !self.stop_reset {
                                self.start_curr = self.last;
                            }
                        }else{
                            if !self.stop_reset {
                                self.start_curr = self.current;
                            }
                        }
                        let mut new = Option::None;
                        mem::swap(&mut self.new_token,&mut new);
                        return new;
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum State{
    Default,
    CarriageReturn,

    NumberLiteral(i32),

    Identifier,

    StringStart,
    StringNormal,
    StringEscape,
    CharLiteralStart,
    CharLiteralNormal(),
    CharLiteralEscape(),

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

    EOF
}

#[derive(Copy, Clone)]
struct FileIndex{
    index: usize,
    index_real: usize,
    line: usize,
    column: usize,
}

impl FileIndex {
    pub fn new() -> FileIndex{
        FileIndex{
            index: 0,
            index_real: 0,
            line: 0,
            column: 0,
        }
    }
}

pub struct Tokenizer<'a> {
    bytes: &'a [u8],
    iterator: Chars<'a>,
    iterations: usize,

    c:char,
    state: State,
    matching: bool,
    stop_reset: bool,
    new_token: Option<Token>,

    current: FileIndex,
    start_curr: FileIndex,
    last: FileIndex,
    escape_start: FileIndex,

    escape_error:bool,
    string:String,
    char_literal:char,
}

trait Test {
    fn from_u8(byte: &[u8]) -> Chars;
}

impl Test for Chars<'_>{
    fn from_u8(byte: &[u8]) -> Chars {
        std::str::from_utf8(byte).expect("").chars()
    }
}

impl<'a> Tokenizer<'a>{
    pub fn new(data:&'a String) -> Tokenizer<'a>{
        Tokenizer{
            bytes: data.as_bytes(),
            iterator: data.chars(),
            iterations: 0,
            state: State::Default,

            current: FileIndex::new(),
            last: FileIndex::new(),
            start_curr: FileIndex::new(),
            escape_start: FileIndex::new(),

            matching: false,
            stop_reset: false,
            new_token: None,
            string: String::new(),
            char_literal: '\0',
            escape_error: false,
            c: '\0'
        }
    }

    pub fn reset(&mut self){
        self.iterations = 0;
        self.state = State::Default;
        self.iterator = Chars::from_u8(self.bytes);


        self.current = FileIndex::new();
        self.last = FileIndex::new();
        self.start_curr = FileIndex::new();
        self.escape_start = FileIndex::new();

        self.matching = false;
        self.stop_reset = false;
        self.new_token = None;
        self.string = String::new();
        self.char_literal = '\0';
        self.escape_error = false;
        self.c = '\0';
    }

    fn create_token(&self, t_type: TokenType) -> Option<Token> {
        let mut temp = Token{t_type,
            index: self.start_curr.index,
            size: (self.current.index - self.start_curr.index),
            size_real: (self.current.index_real - self.start_curr.index_real),
            index_real: self.start_curr.index_real,
            line: self.start_curr.line,
            column: self.start_curr.column
        };
        if self.matching {
            temp.size -= 1;
            temp.size_real -= self.c.len_utf8();
        }
        Option::Some(temp)
    }

    fn create_token_c(size: usize ,index: usize,size_real: usize ,index_real: usize, line: usize, column:usize, t_type: TokenType) -> Option<Token> {
        Option::Some(Token{t_type,
            size,
            index,
            size_real,
            index_real,
            line,
            column
        })
    }

    fn create_number_token(&self, mut num: String) -> Option<Token>{

        let original:String = num.to_string();
        let suffixes = ["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "f32", "f64"];
        let prefixes = ["0x", "0b"];

        let mut suffix: &str = "";
        let mut prefix = "";
        for suf in suffixes{
            if num.ends_with(suf){
                suffix = suf;
                num = num.as_str()[0..num.len() - suf.len()].to_string();
                break;
            }
        }
        for pre in prefixes{
            if num.starts_with(pre){
                prefix = pre;
                num = num.replacen(pre, "", 1);
                break;
            }
        }

        num = num.replace("_","");

        let base;
        match prefix{
            "0x" => base = 16,
            "0b" => base = 2,
            _ => base = 10
        }

        let mut t_type: TokenType;

        macro_rules! parse_to_token{
            ($a: path, $b: ident) => {
                match num.parse::<$a>(){
                    Ok(val) => t_type = TokenType::$b(val),
                    Err(val) => t_type = TokenType::ERROR(val.to_string())
                }
            }
        }

        macro_rules! parse_to_token_base{
            ($a: ident, $b: ident, $c: ident) => {
                match $a::from_str_radix(num.as_str(), $c){
                    Ok(val) => t_type = TokenType::$b(val),
                    Err(val) => t_type = TokenType::ERROR(val.to_string())
                }
            }
        }

        match (suffix, base){
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
            ("f32" | "f64", _) =>
                t_type = TokenType::ERROR("Cannot have non base 10 floating point literal".into()),
            (_, base) => {
                if base == 10{
                    if num.contains("e") || num.contains("E") || num.contains("."){
                        parse_to_token!(f32, F32Literal)
                    }else{
                        parse_to_token_base!(i32, I32Literal, base)
                    }
                }else{
                    parse_to_token_base!(i32, I32Literal, base)
                }
            }
        }
        match t_type{
            TokenType::ERROR(string) =>{
                t_type = TokenType::ERROR(format!("{} for: {}", string, original))
            }
            _ => {}
        }

        self.create_token(t_type)
    }

    fn create_identifier_or_keyword(&self, ident: String) -> Option<Token>{

        let t_type: TokenType;
        match ident.as_str(){
            "void" => t_type = TokenType::VoidKeyword,
            "struct" => t_type = TokenType::StructKeyword,
            "asm" => t_type = TokenType::AsmKeyword,
            "const" => t_type = TokenType::ConstKeyword,
            "static" => t_type = TokenType::StaticKeyword,
            "sizeof" => t_type = TokenType::SizeofKeyword,
            "enum" => t_type = TokenType::EnumKeyword,
            "if" => t_type = TokenType::IfKeyword,
            "else" => t_type = TokenType::ElseKeyword,
            "while" => t_type = TokenType::WhileKeyword,
            "do" => t_type = TokenType::DoKeyword,
            "for" => t_type = TokenType::ForKeyword,
            "return" => t_type = TokenType::ReturnKeyword,
            "break" => t_type = TokenType::BreakKeyword,
            "switch" => t_type = TokenType::SwitchKeyword,
            "case" => t_type = TokenType::CaseKeyword,
            "goto" => t_type = TokenType::GotoKeyword,
            "restrict" => t_type = TokenType::RestrictKeyword,
            "usize" => t_type = TokenType::USizeKeyword,
            "isize" => t_type = TokenType::ISizeKeyword,
            "i8" => t_type = TokenType::I8Keyword,
            "i16" => t_type = TokenType::I16Keyword,
            "i32" => t_type = TokenType::I32Keyword,
            "u8" => t_type = TokenType::U8Keyword,
            "u16" => t_type = TokenType::U16Keyword,
            "u32" => t_type = TokenType::U32Keyword,
            "char" => t_type = TokenType::CharKeyword,
            "bool" => t_type = TokenType::BoolKeyword,
            "true" => t_type = TokenType::BoolLiteral(true),
            "false" => t_type = TokenType::BoolLiteral(false),
            _=> t_type = TokenType::Identifier(ident)
        }

        self.create_token(t_type)
    }

    fn default_reset(&mut self, matching:bool, tok:TokenType){
        self.matching = matching;
        self.new_token = self.create_token(tok);
        self.state = State::Default;
    }

    pub(crate) fn str_from_token(&self, token: &Token) -> String {
        String::from_utf8_lossy(&self.bytes[token.index_real..token.index_real + token.size_real]).to_string()
    }

    pub fn curr_str(&self) -> String {
        if self.matching{
            String::from_utf8_lossy(&self.bytes[self.start_curr.index_real as usize..(self.current.index_real - self.c.len_utf8()) as usize]).to_string()
        }else{
            String::from_utf8_lossy(&self.bytes[self.start_curr.index_real as usize..self.current.index_real as usize]).to_string()
        }
    }
    fn ntm(&mut self){
        self.last = self.current;

        self.current.index += 1;
        self.current.index_real += self.c.len_utf8();
        self.current.column += 1;
        self.stop_reset = false;
    }

    pub fn tokenize(&mut self) -> Vec<Token>{
        let mut tokens = Vec::new();

        loop{
            match self.next(){
                None => {
                    return tokens;
                }
                Some(tok) => {
                    tokens.push(tok)
                }
            }
        }
    }
}