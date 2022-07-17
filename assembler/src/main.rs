use assembler::assembler::assemble;

fn main() {
    let input = "./assembler/res/test.asm";
    let mut output = std::fs::OpenOptions::new()
                            .write(true).create(true)
                            .open("./assembler/res/snake.mxn").unwrap();
    
    match assemble(input, &mut output){
        Ok(report) => {
            println!("Assembled\n{}", report);
        },
        Err(report) => {
            println!("Failed to Assembled\n{}", report);
        },
    }
}
