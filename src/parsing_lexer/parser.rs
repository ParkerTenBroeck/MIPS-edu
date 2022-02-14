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
enum NonTerminal {
    NOTHING,
    Terminal(Token),
    PrimaryExpression(Option<Box<dyn TreeNode>>),
    PostFixExpression(Option<Box<dyn TreeNode>>),
    UnaryExpression(Option<Box<dyn TreeNode>>),
    CastExpression(Option<Box<dyn TreeNode>>),
    MultiplicationExpression(Option<Box<dyn TreeNode>>),
    AdditiveExpression(Option<Box<dyn TreeNode>>),
}

impl Default for NonTerminal {
    fn default() -> Self {
        NonTerminal::NOTHING
    }
}

impl Debug for dyn TreeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeNode")
    }
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

pub trait TreeNode {
    fn test(&self) {}
}

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
        add_reducer!(add_sub_3, 3);
        add_reducer!(add_sub_1, 1);
        add_reducer!(constant_1, 1);

        return tmp;
    }

    fn get_stack_slice(&mut self, size: usize) -> &mut [NonTerminal] {
        let len = self.non_terminal_stack.len();
        if len < size {
            &mut self.non_terminal_stack[0..len]
        } else {
            &mut self.non_terminal_stack[len - size..len]
        }
    }

    pub fn parse(&mut self) -> Result<Box<dyn TreeNode>, &str> {
        let mut reducers = LinkedList::new();
        mem::swap(&mut self.reducer_layers, &mut reducers);

        'main_loop: loop {
            println!("{:?}", self.non_terminal_stack);
            let mut cont = false;

            for reducer in &mut reducers {
                let num = reducer.needed;
                let reduce = reducer.function;
                let size = self.non_terminal_stack.len();

                match reduce(self.get_stack_slice(num)) {
                    ReducerResponse::Reduce(response_val) => {
                        for _ in 0..num - (size - self.non_terminal_stack.len()) {
                            self.non_terminal_stack.pop();
                        }
                        self.non_terminal_stack.push(response_val);
                        //cont |= true;
                        continue 'main_loop;
                    }
                    ReducerResponse::PossibleMatch => {
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

            match self.token_stream.next() {
                None => {
                    cont |= false;
                }
                Some(token) => {
                    self.non_terminal_stack.push(NonTerminal::Terminal(token));
                    cont |= true;
                }
            }

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

    fn steal<T>(item: &mut T) -> T {
        unsafe {
            let mut deref: T = MaybeUninit::zeroed().assume_init();
            mem::swap(item, &mut deref);
            deref
        }
    }

    match_fn!(
        add_sub_3,
        NonTerminal::AdditiveExpression(left),
        NonTerminal::Terminal(
            operator @ Token {
                t_type: TokenType::Plus | TokenType::Minus,
                ..
            },
        ),
        NonTerminal::AdditiveExpression(right),
        {
            NonTerminal::AdditiveExpression(Option::Some(Box::new(BinaryOperator {
                left_size: mem::take(left).unwrap(),
                operator: steal(operator),
                right_size: mem::take(right).unwrap(),
            })))
        }
    );

    match_fn!(add_sub_1, NonTerminal::PrimaryExpression(constant), {
        NonTerminal::AdditiveExpression(Option::Some(mem::take(constant).unwrap()))
    });

    match_fn!(
        constant_1,
        NonTerminal::Terminal(
            token @ Token {
                t_type:
                    TokenType::CharLiteral(_)
                    | TokenType::StringLiteral(_)
                    | TokenType::I32Literal(_)
                    | TokenType::I64Literal(_),
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
