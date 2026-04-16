mod lower;
mod register;

use register::Register;

use std::fmt;

use crate::ir::{
    self,
    frame::{Frame, WORD_SIZE},
    Fragment,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offset {
    pub reg: Register,
    pub offset: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instr {
    // Assembler Directives
    DGlobal {
        label: Label,
    },
    DAlign {
        num_bits: u8,
    },

    // Labels
    Label(Label),

    // Register Instructions
    RInstr {
        name: RInstrName,
        rd: Register,
        rs1: Register,
        rs2: Register,
    },

    // Binary Immediate Instructions
    IInstr2 {
        name: IInstr2Name,
        rd: Register,
        rs: Register,
        imm: i32,
    },

    // Unary Immediate Instructions (li)
    IInstr1 {
        name: IInstr1Name,
        rd: Register,
        imm: i32,
    },

    // Memory Instructions
    Load {
        dst: Register,
        src: Offset,
    },
    Store {
        src: Register,
        dst: Offset,
    },
    LoadAddress {
        dst: Register,
        src: Label,
    },

    // Control Flow
    Call {
        target: Label,
    },
    CallIndirect {
        target: Register,
    },
    Jump {
        label: Label,
    },
    JumpIndirect {
        label: Register,
        offset: i32,
    },
    Branch {
        cc: BranchName,
        rs1: Register,
        rs2: Register,
        target: Label,
    },
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RInstrName {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Xor,
    Sltu,
    Slt,
    Sll,
    Srl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IInstr2Name {
    Addi,
    Xori,
    Slti,
    Slli,
    Srli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IInstr1Name {
    Li,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchName {
    Beq,
    Bne,
    Blt,
    Bge,
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for RInstrName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RInstrName::Add => "add",
            RInstrName::Sub => "sub",
            RInstrName::Mul => "mul",
            RInstrName::Div => "div",
            RInstrName::And => "and",
            RInstrName::Xor => "xor",
            RInstrName::Sltu => "sltu",
            RInstrName::Slt => "slt",
            RInstrName::Sll => "sll",
            RInstrName::Srl => "srl",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for IInstr2Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            IInstr2Name::Addi => "addi",
            IInstr2Name::Xori => "xori",
            IInstr2Name::Slti => "slti",
            IInstr2Name::Slli => "slli",
            IInstr2Name::Srli => "srli",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for IInstr1Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IInstr1Name::Li => write!(f, "li"),
        }
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BranchName::Beq => "beq",
            BranchName::Bne => "bne",
            BranchName::Blt => "blt",
            BranchName::Bge => "bge",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.offset, self.reg)
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instr::DGlobal { label } => {
                write!(f, "\t.globl\t{}", label)
            }
            Instr::DAlign { num_bits } => {
                write!(f, "\t.align\t{}", num_bits)
            }

            Instr::Label(label) => {
                write!(f, "{}:", label)
            }

            Instr::RInstr { name, rd, rs1, rs2 } => {
                write!(f, "\t{}\t{}, {}, {}", name, rd, rs1, rs2)
            }

            Instr::IInstr2 { name, rd, rs, imm } => {
                write!(f, "\t{}\t{}, {}, {}", name, rd, rs, imm)
            }

            Instr::IInstr1 { name, rd, imm } => {
                write!(f, "\t{}\t{}, {}", name, rd, imm)
            }

            Instr::Load { dst, src } => {
                write!(f, "\tld\t{}, {}", dst, src)
            }
            Instr::Store { src, dst } => {
                write!(f, "\tsd\t{}, {}", src, dst)
            }
            Instr::LoadAddress { dst, src } => {
                write!(f, "\tla\t{}, {}", dst, src)
            }

            Instr::Call { target } => {
                write!(f, "\tcall\t{}", target)
            }
            Instr::CallIndirect { target } => {
                write!(f, "\tjalr\t{}", target)
            }
            Instr::Jump { label } => {
                write!(f, "\tj\t{}", label)
            }
            Instr::JumpIndirect { label, offset } => {
                write!(f, "\tjalr\t{}, {}({})", Register::Zero, offset, label)
            }
            Instr::Branch {
                cc,
                rs1,
                rs2,
                target,
            } => {
                write!(f, "\t{}\t{}, {}, {}", cc, rs1, rs2, target)
            }
            Instr::Return => {
                write!(f, "\tret")
            }
        }
    }
}

fn lower_label(l: ir::symbols::Label) -> Label {
    Label(l.0)
}

fn prologue(frame: &Frame) -> Vec<Instr> {
    let mut instrs = Vec::new();
    let frame_size = frame.size();

    instrs.push(Instr::IInstr2 {
        name: IInstr2Name::Addi,
        rd: Register::Sp,
        rs: Register::Sp,
        imm: -frame_size,
    });

    instrs.push(Instr::Store {
        src: Register::Ra,
        dst: Offset {
            reg: Register::Sp,
            offset: frame_size - WORD_SIZE,
        },
    });

    instrs.push(Instr::Store {
        src: Register::Fp,
        dst: Offset {
            reg: Register::Sp,
            offset: frame_size - WORD_SIZE * 2,
        },
    });

    instrs.push(Instr::IInstr2 {
        name: IInstr2Name::Addi,
        rd: Register::Fp,
        rs: Register::Sp,
        imm: frame_size,
    });

    instrs
}

fn epilogue(frame: &Frame) -> Vec<Instr> {
    let mut instrs = Vec::new();
    let frame_size = frame.size();

    instrs.push(Instr::Load {
        dst: Register::Fp,
        src: Offset {
            reg: Register::Sp,
            offset: frame_size - WORD_SIZE * 2,
        },
    });

    instrs.push(Instr::Load {
        dst: Register::Ra,
        src: Offset {
            reg: Register::Sp,
            offset: frame_size - WORD_SIZE,
        },
    });

    instrs.push(Instr::IInstr2 {
        name: IInstr2Name::Addi,
        rd: Register::Sp,
        rs: Register::Sp,
        imm: frame_size,
    });

    instrs.push(Instr::Return);

    instrs
}

fn lower_proc(proc: Fragment) -> Vec<Instr> {
    match proc {
        Fragment::Proc {
            label,
            body,
            mut frame,
        } => {
            let body_instrs = lower::stmts(body, &mut frame);

            let mut final_instrs = Vec::new();
            final_instrs.push(Instr::Label(lower_label(label)));
            final_instrs.extend(prologue(&frame));
            final_instrs.extend(body_instrs);
            final_instrs.extend(epilogue(&frame));

            final_instrs
        }
    }
}

pub type Program = Vec<Instr>;

pub fn select(frags: Vec<Fragment>) -> Program {
    frags.into_iter().flat_map(lower_proc).collect()
}

pub fn emit(program: &Program) -> String {
    let mut out = String::new();

    out.push_str(".text\n");
    out.push_str(".globl main\n\n");

    for instr in program {
        out.push_str(&format!("{instr}\n"));
    }

    out
}
