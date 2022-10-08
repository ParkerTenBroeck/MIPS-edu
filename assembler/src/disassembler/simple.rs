pub fn fmt_reg(reg: usize, use_names: bool) -> &'static str {
    if use_names {
        nammed_regs(reg)
    } else {
        numbered_regs(reg)
    }
}

pub fn nammed_regs(reg: usize) -> &'static str {
    [
        "$zero", "$at", "$v0", "$v1", "$a0", "$a1", "$a2", "$a3", "$t0", "$t1", "$t2", "$t3",
        "$t4", "$t5", "$t6", "$t7", "$s0", "$s1", "$s2", "$s3", "$s4", "$s5", "$s6", "$s7", "$t8",
        "$t9", "$k0", "$k1", "$gp", "$sp", "$fp", "$ra",
    ][reg]
}

pub fn numbered_regs(reg: usize) -> &'static str {
    [
        "$0", "$1", "$2", "$3", "$4", "$5", "$6", "$7", "$8", "$9", "$10", "$11", "$12", "$13",
        "$14", "$15", "$16", "$17", "$18", "$19", "$20", "$21", "$22", "$23", "$24", "$25", "$26",
        "$27", "$28", "$29", "$30", "$31",
    ][reg]
}

pub fn disassemble(opcode: u32, add: u32) -> String {
    if opcode == 0 {
        return "nop".into();
    }

    match opcode >> 26 {
        0b000000 => register_encoding(opcode, add),
        _ => immediate_encoding(opcode, add),
    }
}

fn register_encoding(opcode: u32, _add: u32) -> String {
    let s = (opcode >> 21) & 0b11111;
    let t = (opcode >> 16) & 0b11111;
    let d = (opcode >> 11) & 0b11111;
    let a = (opcode >> 6) & 0b11111;
    let f = opcode & 0b111111;
    let tc = (opcode >> 6) & 0b1111111111;
    let tce = (opcode >> 6) & 0b11111111111111111111;

    match f {
        //arithmatic
        0b100000 =>
        //add
        {
            format!("add   ${}, ${}, ${}", d, s, t)
        }
        0b100001 =>
        //addu
        {
            format!("addu  ${}, ${}, ${}", d, s, t)
        }
        0b100100 =>
        //and
        {
            format!("and   ${}, ${}, ${}", d, s, t)
        }
        0b011010 =>
        //div
        {
            format!("div   ${}, ${}", s, t)
        }
        0b011011 =>
        //divu
        {
            format!("divu  ${}, ${}", s, t)
        }
        0b011000 =>
        //mult
        {
            format!("mult  ${}, ${}", s, t)
        }
        0b011001 =>
        //multu
        {
            format!("multu ${}, ${}", s, t)
        }
        0b100111 =>
        //nor
        {
            format!("nor   ${}, ${}, ${}", d, s, t)
        }
        0b100101 =>
        //or
        {
            format!("or    ${}, ${}, ${}", d, s, t)
        }
        0b000000 =>
        //sll
        {
            format!("sll   ${}, ${}, {}", d, t, a)
        }
        0b000100 =>
        //sllv
        {
            format!("sllv  ${}, ${}, ${}", d, t, s)
        }
        0b000011 =>
        //sra
        {
            format!("sra   ${}, ${}, {}", d, t, a)
        }
        0b000111 =>
        //srav
        {
            format!("srav  ${}, ${}, ${}", d, t, s)
        }
        0b000010 =>
        //srl
        {
            format!("srl   ${}, ${}, {}", d, t, a)
        }
        0b000110 =>
        //srlv
        {
            format!("srlv  ${}, ${}, ${}", d, t, s)
        }
        0b100010 =>
        //sub
        {
            format!("sub   ${}, ${}, ${}", d, s, t)
        }
        0b100011 =>
        //subu
        {
            format!("subu  ${}, ${}, ${}", d, s, t)
        }
        0b100110 =>
        //xor
        {
            format!("xor   ${}, ${}, ${}", d, s, t)
        }

        //comparasin
        0b101010 =>
        //slt
        {
            format!("slt   ${}, ${}, ${}", d, s, t)
        }
        0b101011 =>
        //sltu
        {
            format!("sltu  ${}, ${}, ${}", d, s, t)
        }

        //jump
        0b001001 =>
        //jalr
        {
            format!("jalr  ${}", s)
        }
        0b001000 =>
        //jr
        {
            format!("jr    ${}", s)
        }

        //system
        0b001100 => {
            if tce != 0 {
                format!("syscall  {:#x}", tce)
            } else {
                "syscall".into()
            }
        }
        0b001101 => {
            if tce != 0 {
                format!("break    {:#x}", tce)
            } else {
                "break".into()
            }
        }
        //conditional traps
        0b110100 =>
        //TEQ
        {
            format!("teq   ${}, ${}, {:#X}", s, t, tc)
        }
        0b110000 =>
        //TGE
        {
            format!("tge   ${}, ${}, {:#X}", s, t, tc)
        }
        0b110001 =>
        //TGEU
        {
            format!("tgeu  ${}, ${}, {:#X}", s, t, tc)
        }
        0b110010 =>
        //TIT
        {
            format!("tit   ${}, ${}, {:#X}", s, t, tc)
        }
        0b110011 =>
        //TITU
        {
            format!("titu  ${}, ${}, {:#X}", s, t, tc)
        }
        0b110110 =>
        //TNE
        {
            format!("tne  ${}, ${}, {:#X}", s, t, tc)
        }

        //dataMovement
        0b010000 =>
        //mfhi
        {
            format!("mfhi  ${}", d)
        }
        0b010010 =>
        //mflo
        {
            format!("mflo  ${}", d)
        }
        0b010001 =>
        //mthi
        {
            format!("mthi  ${}", s)
        }
        0b010011 =>
        //mtlo
        {
            format!("mtlo  ${}", s)
        }
        _ => format!("db {:#08x}", opcode),
    }
}
fn immediate_encoding(opcode: u32, add: u32) -> String {
    let o = (opcode >> 26) & 0b111111;
    let s = (opcode >> 21) & 0b11111;
    let t = (opcode >> 16) & 0b11111;
    let sei = ((opcode as i32) << 16) >> 16;
    let zei = opcode & 0xFFFF;
    let b_arr = add.wrapping_add((sei as u32) << 2).wrapping_add(4);

    match o {
        //arthmetic
        0b001000 =>
        //addi
        {
            format!("addi  ${}, ${}, {}", t, s, sei)
        }
        0b001001 =>
        //addiu
        {
            format!("addiu ${}, ${}, {}", t, s, zei)
        }
        0b001100 =>
        //andi
        {
            format!("andi  ${}, ${}, {:#x}", t, s, zei)
        }
        0b001101 =>
        //ori
        {
            format!("ori   ${}, ${}, {:#x}", t, s, zei)
        }
        0b001110 =>
        //xori
        {
            format!("xori  ${}, ${}, {:#x}", t, s, zei)
        }

        //constant manupulating inctructions
        0b001111 =>
        //lhi
        {
            format!("lui   ${}, {:#x}", t, zei)
        }

        //comparison instructions
        0b001010 =>
        //slti
        {
            format!("slti  ${}, ${}, {}", t, s, sei)
        }
        0b001011 =>
        //sltu
        {
            format!("sltu  ${}, ${}, {}", t, s, sei)
        }

        //branch instructions
        0b000100 => {
            //beq
            if t == 0 && s == 0 {
                format!("b     {:#x}", b_arr)
            } else {
                format!("beq   ${}, ${}, {:#x}", t, s, b_arr)
            }
        }
        0b000001 => match t {
            0b00001 => format!("bgez  ${}, ${}, {:#x}", t, s, b_arr),
            0b00000 => format!("bltz  ${}, ${}, {:#x}", t, s, b_arr),
            _ => format!("db {:#08x}", opcode),
        },
        0b000111 =>
        //bgtz
        {
            format!("bgtz  ${}, ${}, {:#x}", t, s, b_arr)
        }
        0b000110 =>
        //blez
        {
            format!("blez  ${}, ${}, {:#x}", t, s, b_arr)
        }
        0b000101 =>
        //bne
        {
            format!("bne   ${}, ${}, {:#x}", t, s, b_arr)
        }

        //load unaliged instructions
        0b100010 => format!("swl   ${}, {}(${})", t, sei, s),
        0b100110 => format!("swr   ${}, {}(${})", t, sei, s),

        //save unaliged instructions
        0b101010 => format!("swl   ${}, {}(${})", t, sei, s),
        0b101110 => format!("swr   ${}, {}(${})", t, sei, s),

        //load instrictions
        0b100000 =>
        //lb
        {
            format!("lb    ${}, {}(${})", t, sei, s)
        }
        0b100100 =>
        //lbu
        {
            format!("lbu   ${}, {}(${})", t, sei, s)
        }
        0b100001 =>
        //lh
        {
            format!("lh    ${}, {}(${})", t, sei, s)
        }
        0b100101 =>
        //lhu
        {
            format!("lhu   ${}, {}(${})", t, sei, s)
        }
        0b100011 =>
        //lw
        {
            format!("lw    ${}, {}(${})", t, sei, s)
        }

        //store instrictions
        0b101000 =>
        //sb
        {
            format!("sb    ${}, {}(${})", t, sei, s)
        }
        0b101001 =>
        //sh
        {
            format!("sh    ${}, {}(${})", t, sei, s)
        }
        0b101011 =>
        //sw
        {
            format!("sw    ${}, {}(${})", t, sei, s)
        }
        _ => jump_encoding(opcode, add),
    }
}
fn jump_encoding(opcode: u32, add: u32) -> String {
    let o = (opcode >> 26) & 0b111111;
    let i = (opcode << 6) >> 6;
    let j_add = add & 0b11110000000000000000000000000000 | (i << 2);

    match o {
        0b000010 => format!("j        {:#x}", j_add),
        0b000011 => format!("jal      {:#x}", j_add),
        _ => format!("db {:#08x}", opcode),
    }
}
