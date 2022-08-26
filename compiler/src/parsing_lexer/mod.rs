pub mod ast;
pub mod highlighter_tokenizer;
pub mod lexer;
pub mod tokenizer;
//pub mod parser;

lalrpop_mod!(pub gen_parser, "/parsing_lexer/gen_parser.rs"); // synthesized by LALRPOP
