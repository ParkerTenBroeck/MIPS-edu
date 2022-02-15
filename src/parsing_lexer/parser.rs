use crate::parsing_lexer::tokenizer::{Token, TokenType, Tokenizer};
use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::iter::Peekable;
use std::mem;
use std::mem::MaybeUninit;

enum ReducerResponse {
    Reduce(NonTerminal),
    PossibleMatch,
    NoMatch,
}

#[derive(Debug)]
#[allow(dead_code)]
enum NonTerminal {
    Terminal(Token),
    PrimaryExpression(Option<Box<dyn TreeNode>>),
    PostFixExpression(Option<Box<dyn TreeNode>>),
    UnaryExpression(Option<Box<dyn TreeNode>>),
    CastExpression(Option<Box<dyn TreeNode>>),
    MultiplicativeExpression(Option<Box<dyn TreeNode>>),
    AdditiveExpression(Option<Box<dyn TreeNode>>),
    ShiftExpression(Option<Box<dyn TreeNode>>),
    RelationalExpression(Option<Box<dyn TreeNode>>),
    EqualityExpression(Option<Box<dyn TreeNode>>),
    AndExpression(Option<Box<dyn TreeNode>>),
    ExclusiveOrExpression(Option<Box<dyn TreeNode>>),
    InclusiveOrExpression(Option<Box<dyn TreeNode>>),
    LogicalAndExpression(Option<Box<dyn TreeNode>>),
    LogicalOrExpression(Option<Box<dyn TreeNode>>),
    AssignmentExpression(Option<Box<dyn TreeNode>>),
    Expression(Option<Box<dyn TreeNode>>),
    Program(Option<Program>),
}

pub struct Parser<'a> {
    token_stream: TokenStream<'a>,
    reducer_layers: LinkedList<Reducer>,
    non_terminal_stack: Vec<NonTerminal>,
}

struct TokenStream<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
}

impl<'a> TokenStream<'a> {
    fn new(tokenizer: Tokenizer<'a>) -> Self {
        let tmp = TokenStream {
            tokenizer: tokenizer.peekable(),
        };

        return tmp;
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.next()
    }
}

struct Reducer {
    function: fn(stack_slice: &mut [NonTerminal]) -> ReducerResponse,
    needed: usize,
}

impl Reducer {
    fn new(function: fn(stack_size: &mut [NonTerminal]) -> ReducerResponse, needed: usize) -> Self {
        Reducer { function, needed }
    }
}

pub trait TreeNode: Debug {
    fn test(&self) {}
}

#[derive(Debug)]
struct Program{

}

impl TreeNode for Program{

}

impl Debug for Token{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.t_type)
    }
}

#[derive(Debug)]
struct UnaryOperator {
    left_size: Box<dyn TreeNode>,
    operator: Token,
    right_size: Box<dyn TreeNode>,
}

impl TreeNode for UnaryOperator {}

#[derive(Debug)]
struct BinaryOperator {
    left_size: Box<dyn TreeNode>,
    operator: Token,
    right_size: Box<dyn TreeNode>,
}

impl TreeNode for BinaryOperator {}

#[derive(Debug)]
struct Constant {
    constant: Token,
}

impl TreeNode for Constant {}

impl<'a> Parser<'a> {
    pub fn new(token_iterator: Tokenizer<'a>) -> Self {
        let mut tmp = Parser {
            token_stream: TokenStream::new(token_iterator),
            reducer_layers: LinkedList::new(),
            non_terminal_stack: Vec::new(),
        };
        macro_rules! add_reducer {
            ($name:ident, $num:literal) => {
                tmp.reducer_layers
                    .push_front(Reducer::new(matching_functions::$name, $num));
            };
        }
        add_reducer!(additive_3, 3);
        add_reducer!(additive_1, 1);
        add_reducer!(multiplicative_3, 3);
        add_reducer!(multiplicative_1, 1);
        add_reducer!(constant_1, 1);

        return tmp;
    }

    fn get_stack_slice(&mut self, size: usize, index: usize) -> &mut [NonTerminal] {
        let len = self.non_terminal_stack.len();
        let mut end = len as isize - index as isize;
        let start = 0;
        if len < size {
            &mut self.non_terminal_stack[0..len]
        } else {
            let mut start = len as isize - size as isize - index as isize;
            if start < 0{
                start = 0;
            }
            &mut self.non_terminal_stack[start as usize..end as usize]
        }
    }

    pub fn parse(&mut self) -> Result<Box<dyn TreeNode>, &str> {
        let mut reducers = LinkedList::new();
        mem::swap(&mut self.reducer_layers, &mut reducers);

        let mut matched = false;
        let mut index:usize = 0;
        'main_loop: loop {
            let mut cont = matched;

            if !matched {
                match self.token_stream.next() {
                    None => {
                        cont |= false;
                    }
                    Some(token) => {
                        self.non_terminal_stack.push(NonTerminal::Terminal(token));
                        cont |= true;
                    }
                }
            }
            matched = false;

            println!("index: {}  = {:?}",index, self.non_terminal_stack);

            for reducer in &mut reducers {
                let num = reducer.needed;
                let reduce = reducer.function;
                let size = self.non_terminal_stack.len();

                match reduce(self.get_stack_slice(num, index)) {
                    ReducerResponse::Reduce(response_val) => {
                        for _ in 0..num - (size - self.non_terminal_stack.len()) {
                            self.non_terminal_stack.pop();
                        }
                        index = 0;
                        self.non_terminal_stack.push(response_val);
                        matched = true;
                        continue 'main_loop;
                    }
                    ReducerResponse::PossibleMatch => {

                        println!("Possible Match");
                        if cont {
                            continue 'main_loop;
                        } else {
                            continue;
                        }
                    }
                    ReducerResponse::NoMatch => {
                        continue;
                    }
                }
            }
            index += 1;
            println!("No match");

            if !cont {
                break;
            }
        }
        self.reducer_layers = reducers;


        if self.non_terminal_stack.len() != 1 {
            Result::Err("Failed to reduce")
        } else {
            match self.non_terminal_stack.pop() {
                Some(NonTerminal::AdditiveExpression(Some(val))) => Result::Ok(val),
                _ => Result::Err("How"),
            }
        }
    }
}

mod matching_functions {
    use super::*;

    macro_rules! generate_match {
    ($obj:ident, $($rest:pat ),+, $inside:block) => {
        generate_match!(0, $obj, $($rest),+, $inside, $($rest ),+);
    };
    ($num:expr, $obj:ident, $a:pat , $($rest:pat ),+, $inside:block, $($all:pat ),+) => {
        if let Option::Some($a) = $obj.get_mut($num){
            generate_match!($num + 1, $obj, $($rest),+, $inside,  $($all ),+);
        }else if $obj.len() == $num{
            return ReducerResponse::PossibleMatch;
        }else{
            return ReducerResponse::NoMatch;
        }

    };
    ($num:expr, $obj:ident, $base:pat , $inside:block, $($all:pat ),+) => {
        if $obj.len() == $num + 1 {
            if let Option::Some($base) = $obj.get_mut($num){
                match $obj{
                    [$($all ),+] => {return ReducerResponse::Reduce($inside);}
                    _ => {return ReducerResponse::NoMatch;}
                }
            }else{
                return ReducerResponse::NoMatch;
            }
        }else{
            return ReducerResponse::NoMatch;
        }

    };
}

    macro_rules! match_fn {
        ($name:ident, $($rest:pat ),+, $inside:block) => {
            #[allow(unused_variables)]
            pub(super) fn $name(stack_slice: &mut [NonTerminal]) -> ReducerResponse{
                generate_match!(stack_slice, $($rest ),+, $inside);
            }
        };
    }


    macro_rules! match_fn_binary{
        ($name:ident,$left:ident,$right:ident, $result:ident, $operator:pat) => {
            match_fn!(
                $name,
                NonTerminal::$left(left),
                NonTerminal::Terminal(
                    operator @ Token {
                        t_type: $operator,
                        ..
                    },
                ),
                NonTerminal::$right(right),
                {
                    NonTerminal::$result(Option::Some(Box::new(BinaryOperator {
                        left_size: mem::take(left).unwrap(),
                        operator: steal(operator),
                        right_size: mem::take(right).unwrap(),
                    })))
                }
            );
        };
    }

    macro_rules! match_fn_transform{
        ($name:ident, $from:ident, $to:ident)  => {
            match_fn!($name, NonTerminal::$from(constant), {
                NonTerminal::$to(Option::Some(mem::take(constant).unwrap()))
            });
        }
    }

    macro_rules! default_expression_match{
        ($name3:ident,$name1:ident,$left:ident,$right:ident, $operator:pat) => {
            match_fn_binary!($name3, $left, $left, $right,$operator);
            match_fn_transform!($name1, $right, $left);
        };
    }


    default_expression_match!(multiplicative_3, multiplicative_1,
        MultiplicativeExpression, PrimaryExpression,
        TokenType::Star | TokenType::Slash | TokenType::Percent);

    default_expression_match!(additive_3, additive_1,
        AdditiveExpression, MultiplicativeExpression,
        TokenType::Plus | TokenType::Minus);


    fn steal<T>(item: &mut T) -> T {
        unsafe {
            let mut deref: T = MaybeUninit::zeroed().assume_init();
            mem::swap(item, &mut deref);
            deref
        }
    }

    match_fn!(
        constant_1,
        NonTerminal::Terminal(
            token @ Token {
                t_type:
                    TokenType::CharLiteral(_)
                    | TokenType::StringLiteral(_)
                    | TokenType::BoolLiteral(_)
                    | TokenType::F32Literal(_)
                    | TokenType::F64Literal(_)
                    | TokenType::I8Literal(_)
                    | TokenType::I16Literal(_)
                    | TokenType::I32Literal(_)
                    | TokenType::I64Literal(_)
                    | TokenType::I128Literal(_)
                    | TokenType::U8Literal(_)
                    | TokenType::U16Literal(_)
                    | TokenType::U32Literal(_)
                    | TokenType::U64Literal(_)
                    | TokenType::U128Literal(_),
                ..
            },
        ),
        {
            NonTerminal::PrimaryExpression(Option::Some(Box::new(Constant {
                constant: steal(token),
            })))
        }
    );
}
