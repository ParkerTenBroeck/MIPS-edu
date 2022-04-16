use assembler::assembler::*;
fn main() {
    if let Result::Err(err) = test(){
        println!("Error {:?}", err);
    }else{
        println!("Assembled");
    }
}

fn test() -> Result<(), Box<dyn std::error::Error>>{
    let mut input = std::fs::OpenOptions::new()
                            .read(true)
                            .open("./assembler/res/test.asm")?;
    let mut output = std::fs::OpenOptions::new()
                            .write(true).create(true)
                            .open("./assembler/res/snake.mxn")?;
    assemble(&mut input, &mut output)?;
    Result::Ok(())
}
