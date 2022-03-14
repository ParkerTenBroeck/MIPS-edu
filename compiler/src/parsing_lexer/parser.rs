use crate::parsing_lexer::tokenizer::{Token, TokenType, Tokenizer};
use std::collections::LinkedList;
use std::fmt::{Debug, Display, Formatter};
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

impl Display for NonTerminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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

pub trait Visitor {
    fn visit_binary_op(&mut self, node: &BinaryOperator) {}
    fn visit_unary_op(&mut self, node: &UnaryOperator) {}
    fn visit_terminal(&mut self, terminal: &Token) {}
    fn visit_unknown(&mut self) {}
}

pub struct PrintVisitor {
    indent: usize,
}

impl PrintVisitor {
    pub fn new() -> Self {
        PrintVisitor { indent: 0 }
    }
    fn print_indent(&self) {
        if self.indent < 1 {
            return;
        }
        for i in 0..self.indent - 1 {
            print!("|  ");
        }
        print!("|--");
    }
}

impl Visitor for PrintVisitor {
    fn visit_binary_op(&mut self, node: &BinaryOperator) {
        self.print_indent();
        self.indent += 1;
        println!("Visited Binary");
        node.left_size.accept(Box::new(self));
        self.visit_terminal(&node.operator);
        node.right_size.accept(Box::new(self));
        self.indent -= 1;
    }
    fn visit_unary_op(&mut self, node: &UnaryOperator) {
        self.print_indent();
        self.indent += 1;
        println!("Visited Unary");
        self.visit_terminal(&node.operator);
        node.expresion.accept(Box::new(self));
        self.indent -= 1;
    }
    fn visit_terminal(&mut self, terminal: &Token) {
        self.print_indent();
        println!("Terminal: {}", terminal.t_type);
    }
    fn visit_unknown(&mut self) {
        self.print_indent();
        println!("Visited Unknown");
    }
}

pub trait TreeNode: Debug + Display {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_unknown();
    }
}

#[derive(Debug)]
struct Program {}

impl TreeNode for Program {}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.t_type)
    }
}

#[derive(Debug)]
pub struct UnaryOperator {
    operator: Token,
    expresion: Box<dyn TreeNode>,
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.expresion)
    }
}

impl TreeNode for UnaryOperator {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_unary_op(self);
    }
}

#[derive(Debug)]
pub struct BinaryOperator {
    left_size: Box<dyn TreeNode>,
    operator: Token,
    right_size: Box<dyn TreeNode>,
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.left_size, self.operator.t_type, self.right_size
        )
    }
}

impl TreeNode for BinaryOperator {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_binary_op(self);
    }
}

#[derive(Debug)]
struct Terminal {
    constant: Token,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Terminal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.constant.t_type)
    }
}

impl TreeNode for Terminal {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_terminal(&self.constant);
    }
}

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
        add_reducer!(expression_1, 1);
        add_reducer!(assignment_1, 1);
        add_reducer!(logical_or_1, 1);
        add_reducer!(logical_or_3, 3);
        add_reducer!(logical_and_1, 1);
        add_reducer!(logical_and_3, 3);
        add_reducer!(or_1, 1);
        add_reducer!(or_3, 3);
        add_reducer!(xor_1, 1);
        add_reducer!(xor_3, 3);
        add_reducer!(and_1, 1);
        add_reducer!(and_3, 3);
        add_reducer!(equality_1, 1);
        add_reducer!(equality_3, 3);
        add_reducer!(relational_1, 1);
        add_reducer!(relational_3, 3);
        add_reducer!(shift_1, 1);
        add_reducer!(shift_3, 3);


        add_reducer!(additive_1, 1);
        add_reducer!(multiplicative_1, 1);

        //add_reducer!(unary_cast_2, 2);

        add_reducer!(additive_3, 3);
        add_reducer!(multiplicative_3, 3);

        add_reducer!(cast_1, 1);

        add_reducer!(assignment_3, 3);

        add_reducer!(unary_1, 1);
        add_reducer!(postfix_1, 1);
        add_reducer!(primary_const_1, 1);
        add_reducer!(primary_ident_1, 1);
        add_reducer!(primary_expr_3, 3);

        add_reducer!(postfix_acc_3, 3);
        add_reducer!(postfix_arr_4, 4);
        add_reducer!(postfix_func_3, 3);
        add_reducer!(postfix_func_4, 4);

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
            if start < 0 {
                start = 0;
            }
            &mut self.non_terminal_stack[start as usize..end as usize]
        }
    }

    pub fn parse(&mut self) -> Result<Box<dyn TreeNode>, &str> {
        let mut reducers = LinkedList::new();
        mem::swap(&mut self.reducer_layers, &mut reducers);

        let mut matched = false;
        let mut index: usize = 0;

        let mut look_ahead_index = index;

        let mut last_match = 0;
        let mut last_match_index = index;
        let mut look_ahead = false;
        let mut last_possible_reducer = 0;
        let mut last_possible_index = 0;

        'main_loop: loop {
            let mut cont = matched;

            if !matched && index == 0{
                match self.token_stream.next() {
                    None => {
                        cont |= false;
                    }
                    Some(token) => {
                        self.non_terminal_stack.push(NonTerminal::Terminal(token));
                        cont |= true;
                        index = 0;
                    }
                }
            }
            matched = false;

            println!("lookahead: {} index: {}  = {:?}",look_ahead, index, self.non_terminal_stack);

            let mut i = -1;
            for reducer in &mut reducers {
                i += 1;
                let num = reducer.needed;
                let reduce = reducer.function;
                let size = self.non_terminal_stack.len();

                match reduce(self.get_stack_slice(num, index)) {
                    ReducerResponse::Reduce(response_val) => {
                        for _ in 0..num - (size - self.non_terminal_stack.len()) {
                            self.non_terminal_stack
                                .remove(self.non_terminal_stack.len() - 1 - index);
                        }
                        let mut size = self.non_terminal_stack.len();

                        self.non_terminal_stack.insert(size - index, response_val);


                        matched = true;
                        //last_match = i;
                        //last_match_index = size;
                        //look_ahead = index > look_ahead_index;
                        //if look_ahead{
                        //    index = look_ahead_index;
                        //}else{
                        index = 0;
                        //last_possible_reducer = -1;
                        //last_possible_index = 0;
                        //}

                        continue 'main_loop;
                    }
                    ReducerResponse::PossibleMatch => {
                        println!("Possible Match");

                        //last_possible_reducer = i;
                        //last_possible_index = index;
                        if cont {
                            //look_ahead = true;
                            //look_ahead_index = self.non_terminal_stack.len();
                            continue 'main_loop;
                        } else {
                            continue;
                        }
                    }
                    ReducerResponse::NoMatch => {
                        //if look_ahead{

                        //}else{
                        //if !look_ahead{
                        //if i <= last_possible_reducer {
                            for s in 1..size {
                                let mut s: isize = s as isize;
                                while last_possible_index > size - s as usize - index{
                                    s -= 1;
                                }
                                match reduce(self.get_stack_slice(s as usize, index)) {
                                    ReducerResponse::Reduce(_) => {}
                                    ReducerResponse::PossibleMatch => {
                                        println!("Possible Match 2");
                                        if cont {
                                            continue 'main_loop;
                                        } else {
                                            continue;
                                        }
                                    }
                                    ReducerResponse::NoMatch => {}
                                }
                            }
                        //}
                        //}
                        //}
                        continue;
                    }
                }
            }
            index += 1;
            //look_ahead_index = index - last_match_index;
            println!("No Match");

            if !cont {
                break;
            }
        }
        self.reducer_layers = reducers;

        if self.non_terminal_stack.len() != 1 {
            Result::Err("Failed to reduce")
        } else {
            match self.non_terminal_stack.pop() {
                Some(NonTerminal::Expression(Some(val))) => Result::Ok(val),
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
        }else if $obj.len() == $num{
            return ReducerResponse::PossibleMatch;
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

    macro_rules! match_fn_binary {
        ($name:ident,$left:ident,$right:ident, $result:ident, $operator:pat) => {
            match_fn!(
                $name,
                NonTerminal::$left(left),
                NonTerminal::Terminal(
                    operator @ Token {
                        t_type: $operator, ..
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

    macro_rules! match_fn_transform {
        ($name:ident, $from:ident, $to:ident) => {
            match_fn!($name, NonTerminal::$from(constant), {
                NonTerminal::$to(Option::Some(mem::take(constant).unwrap()))
            });
        };
    }

    macro_rules! default_expression_match {
        ($name3:ident,$name1:ident,$left:ident,$right:ident, $operator:pat) => {
            match_fn_binary!($name3, $left, $right, $left, $operator);
            match_fn_transform!($name1, $right, $left);
        };
    }

    match_fn_transform!(assignment_1, LogicalOrExpression, AssignmentExpression);
    match_fn!(
        assignment_3,
        NonTerminal::UnaryExpression(left),
        NonTerminal::Terminal(
            operator @ Token {
                t_type:
                    TokenType::Assignment
                    | TokenType::AssignmentAdd
                    | TokenType::AssignmentSub
                    | TokenType::AssignmentMul
                    | TokenType::AssignmentDiv
                    | TokenType::AssignmentMod
                    | TokenType::AssignmentAnd
                    | TokenType::AssignmentOr
                    | TokenType::AssignmentXor
                    | TokenType::AssignmentShiftRight
                    | TokenType::AssignmentShiftLeft,
                ..
            },
        ),
        NonTerminal::LogicalOrExpression(right),
        {
            NonTerminal::AssignmentExpression(Option::Some(Box::new(BinaryOperator {
                left_size: left.take().unwrap(),
                operator: steal(operator),
                right_size: right.take().unwrap(),
            })))
        }
    );

    match_fn_transform!(postfix_1, PrimaryExpression, PostFixExpression);

    match_fn!(
        postfix_arr_4,
        NonTerminal::PostFixExpression(arr),
        NonTerminal::Terminal(
            lbra @ Token {
                t_type: TokenType::LBracket,
                ..
            },
        ),
        NonTerminal::Expression(index),
        NonTerminal::Terminal(
            rbra @ Token {
                t_type: TokenType::RBracket,
                ..
            },
        ),
        { NonTerminal::PostFixExpression(None) }
    );

    match_fn!(
        postfix_func_4,
        NonTerminal::PostFixExpression(arr),
        NonTerminal::Terminal(
            lpar @ Token {
                t_type: TokenType::LPar,
                ..
            },
        ),
        NonTerminal::Expression(index),
        NonTerminal::Terminal(
            rpar @ Token {
                t_type: TokenType::RPar,
                ..
            },
        ),
        { NonTerminal::PostFixExpression(None) }
    );

    match_fn!(
        postfix_func_3,
        NonTerminal::PostFixExpression(func),
        NonTerminal::Terminal(
            lpar @ Token {
                t_type: TokenType::LPar,
                ..
            },
        ),
        NonTerminal::Terminal(
            rpar @ Token {
                t_type: TokenType::RPar,
                ..
            },
        ),
        { NonTerminal::PostFixExpression(None) }
    );

    match_fn!(
        postfix_acc_3,
        NonTerminal::PostFixExpression(item),
        NonTerminal::Terminal(
            lpar @ Token {
                t_type: TokenType::Arrow | TokenType::Dot,
                ..
            },
        ),
        NonTerminal::Terminal(
            rpar @ Token {
                t_type: TokenType::Identifier(_),
                ..
            },
        ),
        { NonTerminal::PostFixExpression(None) }
    );

    match_fn_transform!(unary_1, PostFixExpression, UnaryExpression);
    match_fn!(unary_cast_2,
    NonTerminal::Terminal(operator @ Token{
        t_type: TokenType::Star | TokenType::Minus | TokenType::BitwiseNot, ..
    }), NonTerminal::CastExpression(val),
        {
        NonTerminal::UnaryExpression(Some(Box::new(UnaryOperator{
                expresion: val.take().unwrap(),
                operator: steal(operator),
            })))
    });

    match_fn_transform!(cast_1, UnaryExpression, CastExpression);

    default_expression_match!(
        multiplicative_3,
        multiplicative_1,
        MultiplicativeExpression,
        CastExpression,
        TokenType::Star | TokenType::Slash | TokenType::Percent
    );

    default_expression_match!(
        additive_3,
        additive_1,
        AdditiveExpression,
        MultiplicativeExpression,
        TokenType::Plus | TokenType::Minus
    );

    default_expression_match!(
        shift_3,
        shift_1,
        ShiftExpression,
        AdditiveExpression,
        TokenType::ShiftRight | TokenType::ShiftLeft
    );

    default_expression_match!(
        relational_3,
        relational_1,
        RelationalExpression,
        ShiftExpression,
        TokenType::LessThan
            | TokenType::GreaterThan
            | TokenType::LessThanEq
            | TokenType::GreaterThanEq
    );

    default_expression_match!(
        equality_3,
        equality_1,
        EqualityExpression,
        RelationalExpression,
        TokenType::Equals | TokenType::NotEquals
    );

    default_expression_match!(
        and_3,
        and_1,
        AndExpression,
        EqualityExpression,
        TokenType::Ampersand
    );

    default_expression_match!(
        xor_3,
        xor_1,
        ExclusiveOrExpression,
        AndExpression,
        TokenType::BitwiseXor
    );

    default_expression_match!(
        or_3,
        or_1,
        InclusiveOrExpression,
        ExclusiveOrExpression,
        TokenType::BitwiseOr
    );

    default_expression_match!(
        logical_and_3,
        logical_and_1,
        LogicalAndExpression,
        InclusiveOrExpression,
        TokenType::LogicalAnd
    );

    default_expression_match!(
        logical_or_3,
        logical_or_1,
        LogicalOrExpression,
        LogicalAndExpression,
        TokenType::LogicalOr
    );

    fn steal<T>(item: &mut T) -> T {
        let test = (8 - 4 * 2 + 1) | 125 > 12 && false;
        unsafe {
            let mut deref: T = MaybeUninit::zeroed().assume_init();
            mem::swap(item, &mut deref);
            deref
        }
    }

    match_fn!(
        primary_const_1,
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
            NonTerminal::PrimaryExpression(Option::Some(Box::new(Terminal {
                constant: steal(token),
            })))
        }
    );
    match_fn!(
        primary_ident_1,
        NonTerminal::Terminal(
            token @ Token {
                t_type: TokenType::Identifier(_),
                ..
            },
        ),
        {
            NonTerminal::PrimaryExpression(Option::Some(Box::new(Terminal {
                constant: steal(token),
            })))
        }
    );
    match_fn!(
        primary_expr_3,
        NonTerminal::Terminal(
            lpar @ Token {
                t_type: TokenType::LPar,
                ..
            },
        ),
        NonTerminal::Expression(expression),
        NonTerminal::Terminal(
            rpar @ Token {
                t_type: TokenType::RPar,
                ..
            },
        ),
        { NonTerminal::PrimaryExpression(expression.take()) }
    );

    match_fn_transform!(expression_1, AssignmentExpression, Expression);
}