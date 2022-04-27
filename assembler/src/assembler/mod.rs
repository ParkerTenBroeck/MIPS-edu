use std::{fs::File, error::Error};

use self::assembler::Assembler;

pub mod assembler;
pub mod preprocessor;
pub mod symbol;

#[allow(dead_code)]
#[allow(unused)]
pub fn assemble(input: impl Into<String>, output:&mut  File) -> Result<(), Box<dyn Error>>{
    Assembler::new().assemble(input.into(), output)
}