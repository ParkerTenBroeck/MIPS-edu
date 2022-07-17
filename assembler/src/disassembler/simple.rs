pub fn disassemble(opcode: u32) -> String{
    match opcode >> 26 {
        0b000000 => register_encoding(opcode),
        0b000010|0b000011|0b011010 => jump_encoding(opcode),
        _ => immediate_encoding(opcode),
    }
}
fn register_encoding(opcode: u32) -> String{
    let s = (opcode >> 21) & 0b11111;
    let t = (opcode >> 16) & 0b11111;
    let d = (opcode >> 11) & 0b11111;
    let a = (opcode >> 6) & 0b11111;
    let f = opcode & 0b111111;

    match f {
        //arithmatic
        0b100000 =>  //add
            format!( "add   ${}, ${}, ${}", d, s, t),
        0b100001 =>  //addu
            format!( "addu  ${}, ${}, ${}", d, s, t),
        0b100100 =>  //and
            format!( "and   ${}, ${}, ${}", d, s, t),
        0b011010 =>  //div
            format!( "div   ${}, ${}", s, t),
        0b011011 =>  //divu
            format!( "divu  ${}, ${}", s, t),
        0b011000 =>  //mult
            format!( "mult  ${}, ${}", s, t),
        0b011001 =>  //multu
            format!( "multu ${}, ${}", s, t),
        0b100111 =>  //nor
            format!( "nor   ${}, ${}, ${}", d, s, t),
        0b100101 =>  //or
            format!( "or    ${}, ${}, ${}", d, s, t),
        0b000000 =>  //sll
            format!( "sll   ${}, ${}, {}", d, t, a),
        0b000100 =>  //sllv
            format!( "sllv  ${}, ${}, ${}", d, t, s),
        0b000011 =>  //sra
            format!( "sra   ${}, ${}, {}", d, t, a),
        0b000111 =>  //srav
            format!( "srav  ${}, ${}, ${}", d, t, s),
        0b000010 =>  //srl
            format!( "srl   ${}, ${}, {}", d, t, a),
        0b000110 =>  //srlv
            format!( "srlv  ${}, ${}, ${}", d, t, s),
        0b100010 =>  //sub
            format!( "sub   ${}, ${}, ${}", d, s, t),
        0b100011 =>  //subu
            format!( "subu  ${}, ${}, ${}", d, s, t),
        0b100110 =>  //xor
            format!( "xor   ${}, ${}, ${}", d, s, t),

        //comparasin
        0b101010 =>  //slt
            format!( "slt   ${}, ${}, ${}", d, s, t),
        0b101001 =>  //sltu
            format!( "sltu  ${}, ${}, ${}", d, s, t),

        //jump
        0b001001 =>  //jalr
            format!( "jalr  ${}", s),
        0b001000 =>  //jr
            format!( "jr    ${}", s),

        //dataMovement
        0b010000 =>  //mfhi
            format!( "mfhi  ${}", d),
        0b010010 =>  //mflo
            format!( "mflo  ${}", d),
        0b010001 =>  //mthi
            format!( "mthi  ${}", s),
        0b010011 =>  //mtlo
            format!( "mtlo  ${}", s),
        _ => format!("db {:#08x}", opcode)
    }
}
fn immediate_encoding(opcode: u32) -> String{
    let o = (opcode >> 26) & 0b111111;
    let s = (opcode >> 21) & 0b11111;
    let t = (opcode >> 16) & 0b11111;
    let sei = ((opcode as i32) << 16) >> 16;
    let zei = opcode & 0xFFFF;

    match o {
        //arthmetic
        0b001000 =>  //addi
            format!("addi  ${}, ${}, {}", t, s, sei),
        0b001001 =>  //addiu
            format!("addiu ${}, ${}, {}", t, s, zei),
        0b001100 =>  //andi
            format!("andi  ${}, ${}, {}", t, s, zei),
        0b001101 =>  //ori
            format!("ori   ${}, ${}, {}", t, s, zei),
        0b001110 =>  //xori
            format!("xori  ${}, ${}, {}", t, s, zei),

        //constant manupulating inctructions
        0b011001 =>  //lhi
            format!("lhi   ${}, {}", t, zei),
        0b011000 =>  //llo
            format!("llo   ${}, {}", t, zei),
            
        //comparison instructions
        0b001010 =>  //slti
            format!("slti  ${}, ${}, {}", t, s, sei),

        //branch instructions 
        0b000100 =>  //beq
            format!("beq   ${}, ${}, {}", t, s, sei),
        0b000111 =>  //bgtz
            format!("bgtz  ${}, ${}, {}", t, s, sei),
        0b000110 =>  //blez
            format!("blez  ${}, ${}, {}", t, s, sei),
        0b000101 =>  //bne
            format!("bne   ${}, ${}, {}", t, s, sei),

        //load instrictions
        0b100000 =>  //lb
            format!("lb    ${}, {}(${})", t, sei, s),
        0b100100 =>  //lbu
            format!("lbu   ${}, {}(${})", t, sei, s),
        0b100001 =>  //lh
            format!("lh    ${}, {}(${})", t, sei, s),
        0b100101 =>  //lhu
            format!("lhu   ${}, {}(${})", t, sei, s),
        0b100011 =>  //lw
            format!("lw    ${}, {}(${})", t, sei, s),

        //store instrictions
        0b101000 =>  //sb
            format!("sb    ${}, {}(${})", t, sei, s),
        0b101001 =>  //sh
            format!("sh    ${}, {}(${})", t, sei, s),
        0b101011 =>  //sw
            format!("sw    ${}, {}(${})", t, sei, s),
        _ => format!("db {:#08x}", opcode)
    }
}
fn jump_encoding(opcode: u32) -> String{
    let o = (opcode >> 26) & 0b111111;
    let i = (opcode << 6) >> 6;
    let is = ((opcode as i32) << 6) >> 6;

    match o {
        0b000010 => format!("j     {}", is),
        0b000011 => format!("jal   {}", is),
        0b011010 => format!("trap  {}", i),
        _ => format!("db {:#08x}", opcode)
    }
}