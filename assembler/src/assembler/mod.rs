use std::{fs::File};

use self::assembler::{Assembler, AssemblerReport};

pub mod assembler;
pub mod preprocessor;
pub mod symbol;

#[allow(dead_code)]
#[allow(unused)]
pub fn assemble(input: impl Into<String>, output:&mut  File) -> Result<AssemblerReport, AssemblerReport>{
    Assembler::new().assemble(input.into(), output)
}