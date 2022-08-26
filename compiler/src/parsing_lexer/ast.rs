use crate::parsing_lexer::tokenizer::{Token, TokenType};
use std::collections::LinkedList;
use std::fmt::{Debug, Display, Formatter};

pub trait Visitor {
    fn visit_function_def(&mut self, _node: &FunctionDef) {}
    fn visit_assignment(&mut self, _node: &Assignment) {}
    fn visit_binary_op(&mut self, _node: &BinaryOperator) {}
    fn visit_unary_op(&mut self, _node: &UnaryOperator) {}
    fn visit_terminal(&mut self, _terminal: &Token) {}
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
        for _i in 0..self.indent - 1 {
            print!("|  ");
        }
        print!("|--");
    }
}

impl Visitor for PrintVisitor {
    fn visit_function_def(&mut self, node: &FunctionDef) {
        self.print_indent();
        self.indent += 1;
        println!("Visited Function Definition");
        self.visit_terminal(&node.ident);
        for i in node.statements.iter() {
            i.accept(Box::new(self));
        }
    }
    fn visit_assignment(&mut self, node: &Assignment) {
        self.print_indent();
        self.indent += 1;
        println!("Visited Assignment");

        self.visit_terminal(&node.ident);
        self.visit_terminal(&node.operator);
        node.right_side.accept(Box::new(self));
        self.indent -= 1;
    }
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
        node.expression.accept(Box::new(self));
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
pub struct Program {}

impl TreeNode for Program {
    fn accept(&self, _visitor: Box<&mut dyn Visitor>) {}
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[derive(Debug)]
pub struct FunctionDef {
    ident: Token,
    statements: LinkedList<Box<dyn TreeNode>>,
}

impl FunctionDef {
    pub fn new(ident: Token, statements: LinkedList<Box<dyn TreeNode>>) -> Self {
        FunctionDef { ident, statements }
    }
}

impl TreeNode for FunctionDef {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_function_def(self);
    }
}

impl Display for FunctionDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

//Unary Operator
#[derive(Debug)]
pub struct UnaryOperator {
    operator: Token,
    expression: Box<dyn TreeNode>,
}

impl UnaryOperator {
    pub fn new(operator: Token, expression: Box<dyn TreeNode>) -> Self {
        UnaryOperator {
            operator,
            expression,
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.operator, self.expression)
    }
}

impl TreeNode for UnaryOperator {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_unary_op(self);
    }
}
//Unary Operator

//Function Call
#[derive(Debug)]
pub struct FunctionCall {
    ident: Token,
}

//Function Call

//Binary Operator
#[derive(Debug)]
pub struct BinaryOperator {
    left_size: Box<dyn TreeNode>,
    operator: Token,
    right_size: Box<dyn TreeNode>,
}

impl BinaryOperator {
    pub fn new(
        left_size: Box<dyn TreeNode>,
        operator: Token,
        right_size: Box<dyn TreeNode>,
    ) -> Self {
        BinaryOperator {
            left_size,
            operator,
            right_size,
        }
    }
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
//Binary Operator

//Assignment
#[derive(Debug)]
pub struct Assignment {
    ident: Token,
    operator: Token,
    right_side: Box<dyn TreeNode>,
}

impl Assignment {
    pub fn assignment(ident: Token, operator: Token, right_side: Box<dyn TreeNode>) -> Self {
        Assignment {
            ident,
            operator,
            right_side,
        }
    }
}

impl Display for Assignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TreeNode for Assignment {
    fn accept(&self, visitor: Box<&mut dyn Visitor>) {
        visitor.visit_assignment(self);
    }
}

//Assignment

//Terminal
#[derive(Debug)]
pub struct Terminal {
    constant: Token,
}

impl Terminal {
    pub fn new(constant: Token) -> Self {
        Terminal { constant }
    }
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
//Terminal
