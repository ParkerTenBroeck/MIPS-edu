use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::iter::Peekable;
use std::mem;
use std::mem::MaybeUninit;
use crate::Tokenizer;
use crate::tokenizer::{Token, TokenType};

enum ReducerResponse{
    Reduce(NonTerminal),
    PossibleMatch,
    NoMatch,
}

#[derive(Debug)]
enum NonTerminal {
    NOTHING,
    Terminal(Token),
    AddSub(Option<Box<dyn TreeNode>>),
    Constant(Constant),
}

impl Debug for dyn TreeNode{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TreeNode")
    }
}

pub struct Parser<'a>{

    token_stream: TokenStream<'a>,
    reducer_layers: LinkedList<Box<dyn Reducer>>,
    non_terminal_stack: Vec<NonTerminal>,
}


struct TokenStream<'a>{
    tokenizer: Peekable<Tokenizer<'a>>,

}

impl<'a> TokenStream<'a>{
    fn new(tokenizer: Tokenizer<'a>) -> Self{
        let tmp = TokenStream{
            tokenizer: tokenizer.peekable(),
        };

        return tmp;
    }
}

impl<'a> Iterator for TokenStream<'a>{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokenizer.next()
    }
}

trait Reducer{
    fn reduce(&mut self, stack_slice: &mut [NonTerminal]) -> ReducerResponse;
    fn needed_components(&mut self) -> usize;
}

trait TreeNode{

}

struct NOTHING{

}

struct BinaryOperator{
    left_size: Box<dyn TreeNode>,
    operator: Token,
    right_size: Box<dyn TreeNode>,
}

impl TreeNode for BinaryOperator{

}

#[derive(Debug)]
struct Constant {
    constant: Token,
}

impl TreeNode for Constant{

}

fn steal<T>(item: &mut T) -> T{
    unsafe{
        let mut deref:T =  MaybeUninit::zeroed().assume_init();
        mem::swap(item, &mut deref);
        deref
    }
}

struct AddSubReducer{

}

impl Reducer for AddSubReducer{
    fn reduce(&mut self, stack_slice: &mut [NonTerminal]) -> ReducerResponse {
        return match stack_slice {
            [NonTerminal::AddSub(left),
            NonTerminal::Terminal(operator @ Token { t_type: TokenType::Plus | TokenType::Minus, .. }),
            NonTerminal::AddSub(right)] => {
                let left = steal(left);
                let operator = steal(operator);
                let right = steal(right);
                ReducerResponse::Reduce(NonTerminal::AddSub(Option::Some(Box::new(BinaryOperator {
                    left_size: left.expect(""),
                    operator: operator,
                    right_size: right.expect("")
                }))))
            }
            _ => { ReducerResponse::NoMatch }
        }
    }

    fn needed_components(&mut self) -> usize {
        3
    }
}

struct AddSubReducer1{

}

impl Reducer for AddSubReducer1{
    fn reduce(&mut self, stack_slice: &mut [NonTerminal]) -> ReducerResponse {
        return match stack_slice {
            [NonTerminal::Constant(constant)] => {
                let constant = steal(constant);
                ReducerResponse::Reduce(NonTerminal::AddSub(Option::Some(Box::new(constant))))
            }
            _ => { ReducerResponse::NoMatch }
        }
    }

    fn needed_components(&mut self) -> usize {
        1
    }
}

struct ConstantReducer{
}

impl Reducer for ConstantReducer{
    fn reduce(&mut self, stack_slice: &mut [NonTerminal]) -> ReducerResponse {
        return match stack_slice {
            [NonTerminal::Terminal(token @ Token{t_type:
            TokenType::CharLiteral(_) | TokenType::StringLiteral(_) | TokenType::I32Literal(_) | TokenType::I64Literal(_)
                , ..})] =>{
                ReducerResponse::Reduce(NonTerminal::Constant(Constant{ constant: steal(token) }))
            }
            _ => { ReducerResponse::NoMatch }
        }
    }

    fn needed_components(&mut self) -> usize {
        1
    }
}


impl<'a> Parser<'a>{
    pub fn new(token_iterator: Tokenizer<'a>) -> Self{
        let mut tmp = Parser{
            token_stream: TokenStream::new(token_iterator),
            reducer_layers: LinkedList::new(),
            non_terminal_stack: Vec::new()
        };

        tmp.reducer_layers.push_front(Box::new(AddSubReducer{}));
        tmp.reducer_layers.push_front(Box::new(AddSubReducer1{}));
        tmp.reducer_layers.push_front(Box::new(ConstantReducer{}));

        return tmp;
    }

    fn get_stack_slice(&mut self, size: usize) -> &mut [NonTerminal]{
        let len = self.non_terminal_stack.len();
        if(len < size){
            &mut self.non_terminal_stack[0..len]
        }else{
            &mut self.non_terminal_stack[len-size..len]
        }
    }

    pub fn parse(&mut self){

        let mut reducers = LinkedList::new();
        mem::swap(&mut self.reducer_layers, &mut reducers);

        'main_loop:
        loop{
            println!("{:?}", self.non_terminal_stack);
            let mut cont = false;


            for reducer in &mut reducers{
                let num = reducer.needed_components();
                let size = self.non_terminal_stack.len();

                match reducer.reduce(self.get_stack_slice(num)) {
                    ReducerResponse::Reduce(response_val) => {
                        for _ in 0..num - (size - self.non_terminal_stack.len()) {
                            self.non_terminal_stack.pop();
                        }
                        self.non_terminal_stack.push(response_val);
                        cont |= true;
                        continue 'main_loop;
                    }
                    ReducerResponse::PossibleMatch => {
                        if cont {
                            continue 'main_loop;
                        }else{
                            continue;
                        }
                    }
                    ReducerResponse::NoMatch => {continue;}
                }
            }

            match self.token_stream.next(){
                None => {cont |= false;}
                Some(token) => {
                    self.non_terminal_stack.push(NonTerminal::Terminal(token));
                    cont |= true;
                }
            }

            if !cont{
                break;
            }

        }
        self.reducer_layers = reducers;
    }


}