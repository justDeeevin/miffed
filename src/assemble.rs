use crate::{
    ast::*,
    exec::{Machine, Memory},
};
use std::collections::HashMap;
use thiserror::Error;

pub type Result<T, E = AssembleError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
// TODO: associate a span with this
pub enum AssembleError {
    #[error("Expected register, found immediate")]
    UnexpectedImmediate,
    #[error("Expected immediate, found register")]
    UnexpectedRegister,
    #[error("Given immediate is too large to fit within 16 bits")]
    ImmediateTooLarge,
    #[error("Unidentified label")]
    UnknownLabel,
    #[error("Jump target too large")]
    BadJump,
    #[error("Reading to or storing from $ra is not allowed")]
    BadDoubleRegister,
}

impl TryFrom<Program<'_>> for Machine {
    type Error = AssembleError;

    fn try_from(value: Program<'_>) -> Result<Self> {
        let mut text = Vec::with_capacity(value.text.len());
        let mut labels = value
            .data
            .iter()
            .fold((HashMap::new(), 0), |(mut data, addr), v| {
                data.extend(v.label.map(|l| (l, addr)));
                (data, addr + v.constant.size() as u32)
            })
            .0;
        for (label, instruction) in value.text {
            let addr = text.len() as u32;
            if let Some(label) = label {
                labels.insert(label, addr);
            }
            text.extend(encode_instruction(
                instruction,
                &labels,
                addr,
                value.delay_slot,
            )?);
        }

        Ok(Machine {
            memory: Memory::new(
                text,
                value
                    .data
                    .into_iter()
                    .flat_map(|v| v.constant.bytes())
                    .collect(),
            ),
            registers: [0; 32],
            pc: 0,
            hi: 0,
            lo: 0,
        })
    }
}

fn encode_instruction(
    instruction: Instruction,
    labels: &HashMap<&str, u32>,
    addr: u32,
    delay_slot: bool,
) -> Result<Vec<i32>> {
    let mut out = 0;

    match instruction {
        #[allow(clippy::unusual_byte_groupings)]
        Instruction::Binop { op, dst, lhs, rhs } => {
            out |= rd(dst);
            out |= rs(lhs);
            let mut imm = false;
            out |= match op {
                Binop::Add => 0x20,
                Binop::Addu => 0x21,
                Binop::Addi => {
                    imm = true;
                    8 << 26
                }
                Binop::Addiu => {
                    imm = true;
                    9 << 26
                }
                Binop::And => 0x24,
                Binop::Andi => {
                    imm = true;
                    0xc << 26
                }
                Binop::Div => return Ok(div_pseudo(out, false, dst, rhs)),
                Binop::Divu => return Ok(div_pseudo(out, true, dst, rhs)),
                Binop::Mul => (0x1c << 26) | 6,
                Binop::Mulo => {
                    out &= !0xffff;
                    out |= 0x18;

                    let mut out_vec = Vec::with_capacity(7);

                    let rhs = load_pseudo_rhs(rhs, &mut out_vec);

                    out |= rt(rhs);

                    out_vec.extend([
                        out,
                        // mflo {dst}
                        0x12 | rd(dst),
                        // mfhi $at
                        0x10 | rd(Register::At),
                        // sra {dst}, {dst}, 31
                        0b11111_000011 | rd(dst) | rt(dst),
                        // teq {dst}, $at
                        0x34 | rs(dst) | rt(Register::At),
                        // mflo {dst}
                        0x12 | rd(dst),
                    ]);

                    return Ok(out_vec);
                }
                Binop::Mulou => {
                    out &= !0xffff;
                    out |= 0x19;

                    let mut out_vec = Vec::with_capacity(5);

                    let rhs = load_pseudo_rhs(rhs, &mut out_vec);

                    out |= rt(rhs);

                    out_vec.extend([
                        out,
                        // mflo {dst}
                        0x12 | rd(dst),
                        // mfhi $at
                        0x10 | rd(Register::At),
                        // teq $at, $zero
                        0x34 | rs(Register::At),
                    ]);

                    return Ok(out_vec);
                }
                Binop::Nor => 0x27,
                Binop::Or => 25,
                Binop::Ori => {
                    imm = true;
                    0xd << 26
                }
                Binop::Rem => return rem_pseudo(out, false, dst, rhs),
                Binop::Remu => return rem_pseudo(out, true, dst, rhs),
                Binop::Sll => return shift(out, true, 0, lhs, rhs),
                Binop::Sllv => return shift(out, false, 4, lhs, rhs),
                Binop::Sra => return shift(out, true, 3, lhs, rhs),
                Binop::Srav => return shift(out, false, 7, lhs, rhs),
                Binop::Srl => return shift(out, true, 2, lhs, rhs),
                Binop::Srlv => return shift(out, false, 6, lhs, rhs),
                Binop::Rol => {
                    return Ok(match rhs {
                        Value::Dynamic(rhs) => vec![
                            // subu $at, $zero, {rhs}
                            0x23 | rd(Register::At) | rt(rhs),
                            // srlv $at, {lhs}, $at
                            6 | rd(Register::At) | rt(lhs) | rs(Register::At),
                            // sllv {dst}, {lhs}, {rhs}
                            4 | rd(dst) | rt(lhs) | rs(rhs),
                            // or {dst}, {dst}, $at
                            0x25 | rd(dst) | rs(dst) | rt(Register::At),
                        ],
                        Value::Immediate(shamt) => {
                            vec![
                                // srl $at, {lhs}, {32 - shamt}
                                2 | rd(Register::At) | rt(lhs) | ((32 - shamt) << 6),
                                // sll {dst}, {lhs}, {shamt}
                                rd(dst) | rt(lhs) | (shamt << 6),
                                // or {dst}, {dst}, $at
                                0x25 | rd(dst) | rs(dst) | rt(Register::At),
                            ]
                        }
                    });
                }
                Binop::Ror => {
                    return Ok(match rhs {
                        Value::Dynamic(rhs) => vec![
                            // subu $at, $zero, {rhs}
                            0x23 | rd(Register::At) | rt(rhs),
                            // srlv {dst}, {lhs}, {rhs}
                            6 | rd(dst) | rt(lhs) | rs(rhs),
                            // sllv $at, {lhs}, $at
                            4 | rd(Register::At) | rt(lhs) | rs(Register::At),
                            // or {dst}, {dst}, $at
                            0x25 | rd(dst) | rs(dst) | rt(Register::At),
                        ],
                        Value::Immediate(shamt) => vec![
                            // srl $at, {lhs}, {shamt}
                            2 | rd(Register::At) | rt(lhs) | (shamt << 6),
                            // sll {dst}, {lhs}, {32 - shamt}
                            rd(dst) | rt(lhs) | ((32 - shamt) << 6),
                            // or {dst}, {dst}, $at
                            0x25 | rd(dst) | rs(dst) | rt(Register::At),
                        ],
                    });
                }
                Binop::Sub => 0x22,
                Binop::Subu => 0x23,
                Binop::Xor => 0x26,
                Binop::Xori => {
                    imm = true;
                    0xe << 26
                }
                Binop::Slt => 0x2a,
                Binop::Sltu => 0x2b,
                Binop::Slti => {
                    imm = true;
                    0xa << 26
                }
                Binop::Sltiu => {
                    imm = true;
                    0xb << 26
                }
                Binop::Seq => {
                    return if let Value::Dynamic(rhs) = rhs {
                        out |= 0x23;
                        out |= rt(rhs);
                        Ok(vec![
                            out,
                            // ori $at, $zero, 1
                            (0xd << 26) | rt(Register::At) | 1,
                            // sltu {dst}, {dst}, $at
                            0x2b | rd(dst) | rs(dst) | rt(Register::At),
                        ])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sge => {
                    return if let Value::Dynamic(rhs) = rhs {
                        out |= 0x2a;
                        out |= rt(rhs);
                        Ok(vec![
                            out,
                            // xori {dst}, {dst}, 1
                            (0xe << 26) | rt(dst) | rs(dst) | 1,
                        ])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sgeu => {
                    return if let Value::Dynamic(rhs) = rhs {
                        out |= 0x2b;
                        out |= rt(rhs);
                        Ok(vec![
                            out,
                            // xori {dst}, {dst}, 1
                            (0xe << 26) | rt(dst) | rs(dst) | 1,
                        ])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sgt => {
                    return if let Value::Dynamic(rhs) = rhs {
                        Ok(vec![0x2a | rd(dst) | rs(rhs) | rt(lhs)])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sgtu => {
                    return if let Value::Dynamic(rhs) = rhs {
                        Ok(vec![0x2b | rd(dst) | rs(rhs) | rt(lhs)])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sle => {
                    return if let Value::Dynamic(rhs) = rhs {
                        Ok(vec![
                            // slt $at, {rhs}, {lhs}
                            0x2a | rd(Register::At) | rs(rhs) | rt(lhs),
                            // ori {dst}, $zero, 1
                            (0xd << 26) | rt(dst) | 1,
                            // subu {dst}, {dst}, $at
                            0x23 | rd(dst) | rs(dst) | rt(Register::At),
                        ])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sleu => {
                    return if let Value::Dynamic(rhs) = rhs {
                        Ok(vec![
                            // sltu $at, {rhs}, {lhs}
                            0x2b | rd(Register::At) | rs(rhs) | rt(lhs),
                            // xori {dst}, $at, 1
                            (0xe << 26) | rt(dst) | rs(dst) | 1,
                        ])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Sne => {
                    return if let Value::Dynamic(rhs) = rhs {
                        out |= 0x23;
                        out |= rd(Register::At);
                        out |= rt(rhs);
                        Ok(vec![
                            out,
                            // sltu {dst}, $zero, $at
                            0x2b | rd(dst) | rt(Register::At),
                        ])
                    } else {
                        Err(AssembleError::UnexpectedImmediate)
                    };
                }
                Binop::Movn => 0xb,
                Binop::Movz => 0xa,
            };

            if imm {
                out &= !(0b1111111111 << 16);
                out |= rs(lhs);
                out |= rt(dst);
                if let Value::Immediate(imm) = rhs {
                    if imm > i16::MAX as i32 {
                        Err(AssembleError::ImmediateTooLarge)
                    } else {
                        out |= imm;
                        Ok(vec![out])
                    }
                } else {
                    Err(AssembleError::UnexpectedImmediate)
                }
            } else {
                if let Value::Dynamic(rhs) = rhs {
                    out |= rt(rhs);
                    Ok(vec![out])
                } else {
                    Err(AssembleError::UnexpectedImmediate)
                }
            }
        }
        Instruction::Unop { op, dst, src } => {
            out |= rd(dst);
            let mut hilo = false;
            out |= match op {
                Unop::Abs => {
                    let mut out_vec = Vec::with_capacity(3);
                    let src = load_pseudo_rhs(src, &mut out_vec);
                    out |= rs(src);
                    out |= 0x21;

                    let branch_offset = if delay_slot { 8 } else { 4 };

                    out_vec.extend([
                        out,
                        // bgez {src}, {branch_offset}
                        1 | rs(src) | rt(Register::At) | branch_offset,
                        // subu {dst}, $zero, {src}
                        0x23 | rd(dst) | rt(src),
                    ]);
                    return Ok(out_vec);
                }
                Unop::Clo => 0x21,
                Unop::Clz => 0x20,
                Unop::Div => {
                    hilo = true;
                    0x1a
                }
                Unop::Divu => {
                    hilo = true;
                    0x1b
                }
                Unop::Mult => {
                    hilo = true;
                    0x18
                }
                Unop::Multu => {
                    hilo = true;
                    0x19
                }
                Unop::Madd => {
                    hilo = true;
                    0x1c << 26
                }
                Unop::Maddu => {
                    hilo = true;
                    (0x1c << 26) | 1
                }
                Unop::Move => 0x21 | rs(Register::Zero),
                Unop::Msub => {
                    hilo = true;
                    (0x1c << 26) | 4
                }
                Unop::Msubu => {
                    hilo = true;
                    (0x1c << 26) | 5
                }
                Unop::Neg => rs(Register::Zero) | 0x22,
                Unop::Negu => rs(Register::Zero) | 0x23,
                Unop::Not => rs(Register::Zero) | 0x27,
                Unop::Lui => {
                    return if let Value::Immediate(imm) = src {
                        Ok(vec![(0xf << 26) | rt(dst) | imm])
                    } else {
                        Err(AssembleError::UnexpectedRegister)
                    };
                }
                Unop::Li => return li(dst, src),
            };

            if let Value::Dynamic(src) = src {
                if hilo {
                    out &= !(0xfffff << 6);
                    out |= rs(dst);
                } else {
                    out |= rs(src);
                }

                Ok(vec![out])
            } else {
                Err(AssembleError::UnexpectedImmediate)
            }
        }
        Instruction::MoveHiLo { reg, hi, to } => {
            out |= rd(reg);
            let suffix = 0x10 + (!hi as i32 * 2) + to as i32;
            out |= suffix;

            Ok(vec![out])
        }
        Instruction::La { dst, index } => {
            let offset = resolve_address(index.offset, labels)?;
            let mut out_vec = li(dst, Value::Immediate(offset))?;
            if let Some(addr) = index.addr {
                // add {dst}, {dst}, {addr}
                out_vec.push(0x20 | rd(dst) | rs(dst) | rt(addr));
            }

            Ok(out_vec)
        }
        Instruction::Mem {
            op,
            reg,
            index,
            width,
        } => {
            let mut offset = resolve_address(index.offset, labels)?;
            let mut opcode = 20;
            let mut double = false;
            opcode += match width {
                Width::Byte => 0,
                Width::ByteUnaligned => 4,
                Width::Half => 1,
                Width::HalfUnaligned => 5,
                Width::Word => 3,
                Width::WordLeft => 2,
                Width::WordRight => 6,
                Width::Double => {
                    double = true;
                    3
                }
            };
            if op == MemOp::Store {
                opcode += 8;
            }
            if let Some(addr) = index.addr {
                out |= rs(addr);
            }
            out |= opcode;

            let mut out_vec = vec![out | rt(reg) | offset];
            if double {
                if reg == Register::Ra {
                    return Err(AssembleError::BadDoubleRegister);
                }
                offset += 4;
                // lw {reg + 1}, {addr}({offset + 4})
                out_vec.push(out | (rt(reg) + rt(Register::At)) | (offset + 4));
            }

            Ok(out_vec)
        }
        Instruction::Branch { cond, target } => {
            let offset = match target {
                Address::Literal(target) => target as i32,
                Address::Label(target) => {
                    *labels.get(target).ok_or(AssembleError::UnknownLabel)? as i32 - addr as i32
                }
            };
            assert!(
                i16::try_from(offset).is_ok(),
                "Branch offset too large: {offset}"
            );
            out |= offset;

            match cond {
                Condition::Binary { cond, mut lhs, rhs } => {
                    let mut out_vec = Vec::with_capacity(2);
                    let mut rhs = load_pseudo_rhs(rhs, &mut out_vec);

                    out |= rs(lhs);

                    let (branch_opcode, pseudo) = match cond {
                        BinCond::Eq => (4, None),
                        BinCond::Ne => (5, None),
                        BinCond::Ge => (4, Some((false, false))),
                        BinCond::Geu => (4, Some((true, false))),
                        BinCond::Gt => (5, Some((false, true))),
                        BinCond::Gtu => (5, Some((true, true))),
                        BinCond::Le => (4, Some((false, true))),
                        BinCond::Leu => (4, Some((true, true))),
                        BinCond::Lt => (5, Some((false, false))),
                        BinCond::Ltu => (5, Some((true, false))),
                    };

                    if let Some((unsigned, swap)) = pseudo {
                        let cmp_opcode = 0x2a + unsigned as i32;
                        if swap {
                            std::mem::swap(&mut lhs, &mut rhs);
                        }
                        out_vec.extend([
                            cmp_opcode | rd(Register::At) | rs(lhs) | rt(rhs),
                            (branch_opcode << 26) | rs(Register::At) | offset,
                        ])
                    } else {
                        if rhs == Register::At {
                            return Err(AssembleError::UnexpectedImmediate);
                        }
                        out |= rt(rhs);
                        out |= branch_opcode << 26;
                        out_vec.push(out);
                    }

                    Ok(out_vec)
                }
                Condition::Unary { cond, src, link } => {
                    let (opcode, mut fncode) = match cond {
                        UnCond::Gez => (1, 1),
                        UnCond::Gtz => (7, 0),
                        UnCond::Lez => (6, 0),
                        UnCond::Ltz => (1, 0),
                        UnCond::Eqz => (4, 0),
                        UnCond::Nez => (5, 0),
                    };
                    out |= opcode << 26;
                    if link {
                        fncode += 0x10;
                    }
                    out |= fncode << 16;
                    out |= rs(src);

                    Ok(vec![out])
                }
            }
        }
        Instruction::Jump { target, link } => {
            match target {
                Value::Dynamic(target) => {
                    out |= rs(target);
                    let mut fncode = 8;
                    if let Some(link) = link {
                        out |= rd(link);
                        fncode += 1;
                    }
                    out |= fncode
                }
                Value::Immediate(target) => {
                    let target = resolve_address(target, labels)?;
                    if target.leading_zeros() >= (32 - 26) {
                        return Err(AssembleError::BadJump);
                    }
                    let opcode = 2 + link.is_some() as i32;
                    out |= target;
                    out |= opcode << 26;
                }
            }

            Ok(vec![out])
        }
        Instruction::Trap { cond, lhs, rhs } => {
            out |= rs(lhs);
            match rhs {
                Value::Dynamic(rhs) => {
                    out |= rt(rhs);
                    out |= 0x30 + cond as i32;
                }
                Value::Immediate(rhs) => {
                    out |= 1 << 26;
                    out |= rhs;
                    out |= (8 + cond as i32) << 16;
                }
            }

            Ok(vec![out])
        }
        Instruction::Syscall => Ok(vec![0xc]),
        Instruction::Nop => Ok(vec![0]),
        Instruction::Break(code) => Ok(vec![0xd | (code << 6)]),
    }
}

fn resolve_address(addr: Address<'_>, labels: &HashMap<&str, u32>) -> Result<i32> {
    Ok(match addr {
        Address::Label(label) => *labels.get(label).ok_or(AssembleError::UnknownLabel)? as i32,
        Address::Literal(addr) => addr as i32,
    })
}

fn li(dst: Register, src: Value) -> Result<Vec<i32>> {
    if let Value::Immediate(imm) = src {
        let mut out_vec = Vec::with_capacity(1);
        if imm > i16::MAX as i32 {
            out_vec.push((0xf << 26) | rt(dst) | (imm >> 16));
        }
        let rs = rs(if imm > i16::MAX as i32 {
            dst
        } else {
            Register::Zero
        });
        out_vec.push((0xd << 26) | rt(dst) | rs | (imm & 0xffff));
        Ok(out_vec)
    } else {
        Err(AssembleError::UnexpectedRegister)
    }
}

const fn rs(r: Register) -> i32 {
    (r as i32) << 21
}

const fn rt(r: Register) -> i32 {
    (r as i32) << 16
}

const fn rd(r: Register) -> i32 {
    (r as i32) << 11
}

fn shift(mut out: i32, variable: bool, opcode: i32, lhs: Register, rhs: Value) -> Result<Vec<i32>> {
    out |= (lhs as i32) << 16;
    if variable {
        let Value::Immediate(shamt) = rhs else {
            return Err(AssembleError::UnexpectedRegister);
        };
        out |= shamt << 6;
    } else {
        let Value::Dynamic(rhs) = rhs else {
            return Err(AssembleError::UnexpectedImmediate);
        };
        out &= !(0b11111 << 21);
        out |= (rhs as i32) << 21;
    }
    Ok(vec![out | opcode])
}

#[allow(clippy::unusual_byte_groupings)]
fn rem_pseudo(mut out: i32, unsigned: bool, dst: Register, rhs: Value) -> Result<Vec<i32>> {
    out &= !0xffff;
    out |= 0x1a + unsigned as i32;
    let Value::Dynamic(rhs) = rhs else {
        return Err(AssembleError::UnexpectedImmediate);
    };

    out |= (rhs as i32) << 16;

    Ok(vec![
        out,
        // mflo {dst}
        0x12 | rd(dst),
    ])
}

#[allow(clippy::unusual_byte_groupings)]
fn load_pseudo_rhs(value: Value, out: &mut Vec<i32>) -> Register {
    match value {
        Value::Dynamic(rhs) => rhs,
        Value::Immediate(rhs) => {
            if rhs <= i16::MAX as i32 {
                // ori $at, $zero, {value}
                out.extend([(13 << 26) | rt(Register::At) | rhs])
            } else {
                out.extend([
                    // lui $at, {value >> 16}
                    (15 << 26) | (rhs >> 16),
                    // ori $at, $at, {value & 0xffff}
                    (13 << 26) | rt(Register::At) | rs(Register::At) | (rhs & 0xffff),
                ])
            }
            Register::At
        }
    }
}

#[allow(clippy::unusual_byte_groupings)]
fn div_pseudo(mut out: i32, unsigned: bool, dst: Register, rhs: Value) -> Vec<i32> {
    out &= !0xffff;
    out |= 0x1a + unsigned as i32;

    let mut out_vec = Vec::with_capacity(4);

    let rhs = load_pseudo_rhs(rhs, &mut out_vec);

    out |= rt(rhs);

    out_vec.extend([
        // teqi {rhs}, 0
        (1 << 26) | rs(rhs),
        out,
        // mflo {dst}
        0x12 | rd(dst),
    ]);
    out_vec
}
