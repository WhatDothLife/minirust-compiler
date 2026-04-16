use super::{BranchName, IInstr1Name, IInstr2Name, Instr, Label, Offset, RInstrName, Register};
use crate::ir::frame::Frame;
use crate::ir::symbols::Temp;
use crate::ir::{BinOp, Expr, RelOp, Stmt};

pub fn stmts(stmts: Vec<Stmt>, frame: &mut Frame) -> Vec<Instr> {
    let mut muncher = Muncher::new(frame);
    muncher.munch_all(stmts);
    muncher.finish()
}

struct Muncher<'a> {
    frame: &'a mut Frame,
    instrs: Vec<Instr>,
}

impl<'a> Muncher<'a> {
    fn new(frame: &'a mut Frame) -> Self {
        Self {
            frame,
            instrs: Vec::new(),
        }
    }

    fn finish(self) -> Vec<Instr> {
        self.instrs
    }

    fn emit(&mut self, instr: Instr) {
        self.instrs.push(instr);
    }

    pub fn munch_all(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            self.munch_stmt(stmt);
        }
    }

    fn munch_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Move(
                Expr::Mem(box Expr::BinOp(BinOp::Add, e1, box Expr::Const(i)))
                | Expr::Mem(box Expr::BinOp(BinOp::Add, box Expr::Const(i), e1)),
                src,
            ) => {
                let r_src = self.munch_exp(src);
                let tmp_src = Temp::new();
                self.store_temp(r_src, tmp_src);

                let r_base = self.munch_exp(*e1);
                self.load_temp(tmp_src, Register::T1);

                self.emit(Instr::Store {
                    src: Register::T1,
                    dst: Offset {
                        reg: r_base,
                        offset: i as i32,
                    },
                });
            }

            // MOVE(MEM(CONST(i)), e2)
            Stmt::Move(Expr::Mem(box Expr::Const(i)), e2) => {
                let r_val = self.munch_exp(e2);
                self.emit(Instr::Store {
                    src: r_val,
                    dst: Offset {
                        reg: Register::Zero,
                        offset: i as i32,
                    },
                });
            }

            // MOVE(MEM(e1), e2)
            Stmt::Move(Expr::Mem(box e1), e2) => {
                let r_val = self.munch_exp(e2);
                let tmp_val = Temp::new();
                self.store_temp(r_val, tmp_val);

                let r_addr = self.munch_exp(e1);
                self.load_temp(tmp_val, Register::T1);

                self.emit(Instr::Store {
                    src: Register::T1,
                    dst: Offset {
                        reg: r_addr,
                        offset: 0,
                    },
                });
            }

            Stmt::Move(Expr::Temp(t), e2) => {
                let r_src = self.munch_exp(e2);

                if let Some(abi_reg) = get_abi_reg(t) {
                    self.emit(Instr::IInstr2 {
                        name: IInstr2Name::Addi,
                        rd: abi_reg,
                        rs: r_src,
                        imm: 0,
                    });
                } else {
                    self.store_temp(r_src, t);
                }
            }

            Stmt::CJump(op, e1, e2, t_lab, f_lab) => {
                let r1 = self.munch_exp(e1);
                let tmp1 = Temp::new();
                self.store_temp(r1, tmp1);

                let r2 = self.munch_exp(e2);
                let tmp2 = Temp::new();
                self.store_temp(r2, tmp2);

                self.load_temp(tmp1, Register::T1);
                self.load_temp(tmp2, Register::T2);

                let (cc, left, right) = match op {
                    RelOp::Eq => (BranchName::Beq, Register::T1, Register::T2),
                    RelOp::Ne => (BranchName::Bne, Register::T1, Register::T2),
                    RelOp::Lt => (BranchName::Blt, Register::T1, Register::T2),
                    RelOp::Ge => (BranchName::Bge, Register::T1, Register::T2),

                    RelOp::Gt => (BranchName::Blt, Register::T2, Register::T1),
                    RelOp::Le => (BranchName::Bge, Register::T2, Register::T1),
                    _ => todo!("Other RelOps"),
                };

                self.emit(Instr::Branch {
                    cc,
                    rs1: left,
                    rs2: right,
                    target: Label(t_lab.0),
                });

                self.emit(Instr::Jump {
                    label: Label(f_lab.0),
                });
            }

            Stmt::Jump(Expr::Name(l), _) => self.emit(Instr::Jump { label: Label(l.0) }),
            Stmt::Label(l) => self.emit(Instr::Label(Label(l.0))),
            Stmt::Expr(e) => {
                self.munch_exp(e);
            }
            _ => panic!("Unexpected statement: {:?}", stmt),
        }
    }

    fn munch_exp(&mut self, expr: Expr) -> Register {
        match expr {
            Expr::Mem(box Expr::BinOp(BinOp::Add, e1, box Expr::Const(i)))
            | Expr::Mem(box Expr::BinOp(BinOp::Add, box Expr::Const(i), e1)) => {
                let r1 = self.munch_exp(*e1);
                self.emit(Instr::Load {
                    dst: Register::T0,
                    src: Offset {
                        reg: r1,
                        offset: i as i32,
                    },
                });
                Register::T0
            }

            Expr::Mem(box Expr::Const(i)) => {
                self.emit(Instr::Load {
                    dst: Register::T0,
                    src: Offset {
                        reg: Register::Zero,
                        offset: i as i32,
                    },
                });
                Register::T0
            }

            Expr::Mem(box e1) => {
                let r_addr = self.munch_exp(e1);
                self.emit(Instr::Load {
                    dst: Register::T0,
                    src: Offset {
                        reg: r_addr,
                        offset: 0,
                    },
                });
                Register::T0
            }

            Expr::BinOp(BinOp::Add, e1, box Expr::Const(i))
            | Expr::BinOp(BinOp::Add, box Expr::Const(i), e1) => {
                let r1 = self.munch_exp(*e1);
                self.emit(Instr::IInstr2 {
                    name: IInstr2Name::Addi,
                    rd: Register::T0,
                    rs: r1,
                    imm: i as i32,
                });
                Register::T0
            }

            Expr::Const(i) => {
                self.emit(Instr::IInstr1 {
                    name: IInstr1Name::Li,
                    rd: Register::T0,
                    imm: i as i32,
                });
                Register::T0
            }

            Expr::BinOp(op, e1, e2) => {
                let name = match op {
                    BinOp::Sub => RInstrName::Sub,
                    BinOp::Mul => RInstrName::Mul,
                    BinOp::Div => RInstrName::Div,
                    BinOp::And => RInstrName::And,
                    BinOp::Xor => RInstrName::Xor,
                    BinOp::Add => RInstrName::Add,
                    _ => todo!("Other BinOps"),
                };
                self.munch_binop(name, *e1, *e2)
            }

            Expr::Temp(t) => {
                if let Some(abi_reg) = get_abi_reg(t) {
                    abi_reg
                } else {
                    self.load_temp(t, Register::T0);
                    Register::T0
                }
            }

            Expr::Call(func_expr, args) => {
                let mut arg_temps = Vec::new();

                for arg in args {
                    let r_arg = self.munch_exp(arg);
                    let t_arg = Temp::new();
                    self.store_temp(r_arg, t_arg);
                    arg_temps.push(t_arg);
                }

                for (i, t_arg) in arg_temps.into_iter().enumerate() {
                    if i < 8 {
                        let target_reg = [
                            Register::A0,
                            Register::A1,
                            Register::A2,
                            Register::A3,
                            Register::A4,
                            Register::A5,
                            Register::A6,
                            Register::A7,
                        ][i];

                        self.load_temp(t_arg, target_reg);
                    } else {
                        panic!("Too many arguments!");
                    }
                }

                match *func_expr {
                    Expr::Name(l) => self.emit(Instr::Call { target: Label(l.0) }),
                    other => {
                        let r_func = self.munch_exp(other);
                        self.emit(Instr::CallIndirect { target: r_func });
                    }
                }

                Register::A0
            }

            Expr::Name(l) => {
                self.emit(Instr::LoadAddress {
                    dst: Register::T0,
                    src: Label(l.0),
                });
                Register::T0
            }

            Expr::ESeq(_, _) => unreachable!("ESeq should have been eliminated"),
        }
    }

    fn munch_binop(&mut self, name: RInstrName, e1: Expr, e2: Expr) -> Register {
        let r1 = self.munch_exp(e1);
        let tmp = Temp::new();
        self.store_temp(r1, tmp);

        let r2 = self.munch_exp(e2);
        self.load_temp(tmp, Register::T1);

        self.emit(Instr::RInstr {
            name,
            rd: Register::T0,
            rs1: Register::T1,
            rs2: r2,
        });
        Register::T0
    }

    fn store_temp(&mut self, src: Register, temp: Temp) {
        let offset = self.frame.get_offset(temp);
        self.emit(Instr::Store {
            src,
            dst: Offset {
                reg: Register::Fp,
                offset,
            },
        });
    }

    fn load_temp(&mut self, temp: Temp, dst: Register) {
        let offset = self.frame.get_offset(temp);
        self.emit(Instr::Load {
            dst,
            src: Offset {
                reg: Register::Fp,
                offset,
            },
        });
    }
}

fn get_abi_reg(t: Temp) -> Option<Register> {
    match t.0 {
        0 => Some(Register::Fp),
        1 => Some(Register::Sp),
        2 | 3 => Some(Register::A0),
        4..=10 => Some(match t.0 {
            4 => Register::A1,
            5 => Register::A2,
            6 => Register::A3,
            7 => Register::A4,
            8 => Register::A5,
            9 => Register::A6,
            10 => Register::A7,
            _ => unreachable!(),
        }),
        _ => None,
    }
}
