use assembler::assembler::*;
fn main() {
    let mut input = std::fs::File::open("./assembler/res/snake.asm").unwrap();
    let mut output = std::fs::File::open("./assembler/res/snake.mxn").unwrap();
    match assemble(&mut input, &mut output){
        Ok(_) => println!("assembled"),
        Err(err) => println!("{}", err),
    }
}
