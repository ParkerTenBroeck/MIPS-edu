use std::fs::File;

#[allow(clippy::module_inception)]
mod assembler;
pub use self::assembler::*;
pub mod preprocessor;
pub mod symbol;

#[allow(dead_code)]
#[allow(unused)]
pub fn assemble(
    input: impl Into<String>,
    output: &mut File,
) -> Result<AssemblerReport, AssemblerReport> {
    Assembler::new().assemble(input.into(), output)
}
