use assembler::assembler::assemble;

fn main() {
    if let Result::Err(err) = test(){
        println!("Error {:?}", err);
    }else{
        println!("Assembled");
    }
}

fn test() -> Result<(), Box<dyn std::error::Error>>{

    let mut output = std::fs::OpenOptions::new()
                            .write(true).create(true)
                            .open("./assembler/res/snake.mxn")?;
    assemble("./assembler/res/test.asm", &mut output)?;
    Result::Ok(())
}
