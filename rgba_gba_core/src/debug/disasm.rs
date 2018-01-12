// disasm.rs --- 
// 
// Filename: disasm.rs
// Author: Louise <louise>
// Created: Mon Jan  8 14:49:33 2018 (+0100)
// Last-Updated: Fri Jan 12 21:45:15 2018 (+0100)
//           By: Louise <louise>
// 

const CONDITIONS: [&str; 16] = [
    "eq", "ne", "cs", "cc",
    "mi", "pl", "vs", "vc",
    "hi", "ls", "ge", "lt",
    "gt", "le", "",   "nv"
];

const SHIFTS: [&str; 5] = [
    "lsl", "lsr", "asr", "ror", "rrx"
];

const ARM_INSTRS: [(u32, u32, &str); 25] = [
    // Branches
    (0x0F000000, 0x0A000000, "b%c %o"),
    (0x0F000000, 0x0B000000, "bl%c %o"),
    (0x0FFFFFF0, 0x012FFF10, "bx%c %r0"),
    // PSR Transfer
    (0x0FBF0FFF, 0x010F0000, "msr%c %r3, %p"),
    (0x0DB0F000, 0x0120F000, "msr%c %p, %i"),
    // Multiply
    (0x0FE000F0, 0x00000090, "mul%s%c %r4, %r0, %r2"),
    (0x0FE000F0, 0x00200090, "mla%s%c %r4, %r0, %r2, %r3"),
    // Multiply long
    (0x0FA000F0, 0x00800090, "%umull%s%c %r3, %r4, %r0, %r2"),
    (0x0FA000F0, 0x00A00090, "%umlal%s%c %r3, %r4, %r0, %r2"),
    // ALU
    (0x0DE00000, 0x00000000, "and%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00200000, "eor%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00400000, "sub%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00600000, "rsb%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00800000, "add%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00A00000, "adc%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00C00000, "sbc%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x00E00000, "rsc%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x01000000, "tst%c %r4, %i"),
    (0x0DE00000, 0x01200000, "teq%c %r4, %i"),
    (0x0DE00000, 0x01400000, "cmp%c %r4, %i"),
    (0x0DE00000, 0x01600000, "cmn%c %r4, %i"),
    (0x0DE00000, 0x01800000, "orr%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x01A00000, "mov%s%c %r3, %i"),
    (0x0DE00000, 0x01C00000, "bic%s%c %r3, %r4, %i"),
    (0x0DE00000, 0x01E00000, "mvn%s%c %r3, %i"),
];

pub fn disasm_arm(offset: u32, instr: u32) -> String {
    let mut dis = String::new();
    
    for &(mask, res, disasm) in ARM_INSTRS.iter() {
        if instr & mask == res {
            let mut it = disasm.chars();

            while let Some(c) = it.next() {
                if c == '%' {
                    match it.next() {
                        Some('c') =>
                            dis.push_str(CONDITIONS[(instr >> 28) as usize]),
                        Some('p') => dis.push_str(if instr & 0x400000 != 0 { "spsr" } else { "cpsr" }),
                        Some('r') => {
                            let shifted = instr >> (it.next().unwrap().to_digit(10).unwrap() << 2);
                            
                            dis.push_str(&format!("r{}", shifted & 0xF))
                        }
                        Some('s') => if instr & 0x100000 != 0 { dis.push('s'); },
                        Some('u') => if instr & 0x400000 != 0 { dis.push('u'); },
                        Some('i') => {
                            if instr & 0x02000000 != 0 {
                                let imm = instr & 0xFF;
                                let rot = (instr & 0xF00) >> 7;

                                dis.push_str(&format!("0x{:08x}", imm.rotate_right(rot)));
                            } else {
                                let rm = instr & 0xF;
                                let mut shift = (instr & 0x60) >> 5;

                                dis.push_str(&format!("r{}", rm));
                                
                                if instr & 0x10 != 0 {
                                    dis.push(' ');
                                    dis.push_str(SHIFTS[shift as usize]);
                                    dis.push_str(&format!(" r{}", (instr & 0xF00) >> 8));
                                } else {
                                    let mut amount = (instr & 0xF80) >> 7;

                                    if amount == 0 && shift == 3 { shift = 4 }
                                    if amount == 0 { amount = 32; }

                                    if amount != 32 || shift != 0 {
                                        dis.push(' ');
                                        dis.push_str(SHIFTS[shift as usize]);
                                        dis.push_str(&format!(" #{}", amount));
                                    }
                                }
                            }
                        }
                        Some('o') => {
                            let mut off = instr & 0xFFFFFF;
                            
			    if off & 0x800000 != 0 {
				off |= 0xff000000;
			    }
                            
                            off <<= 2;
                            
                            dis.push_str(
                                &format!("0x{:x}",
                                         offset as i32 + off as i32 + 8
                                )
                            )
                        },
                        Some(e) => println!("{}", e),
                        _ => panic!()
                    }
                } else {
                    dis.push(c);
                }
            }
        }
    }

    dis
}

pub fn disasm_thumb(instr: u16) -> String {
    format!("")
}
