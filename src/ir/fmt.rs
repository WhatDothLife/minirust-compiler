use crate::pretty::{space, Pretty};

use super::{Expr, Fragment, Program, Stmt};

impl Pretty for Expr {
    fn pretty(&self, indent: usize) -> String {
        match self {
            Expr::Const(n) => format!("Const({})", n),
            Expr::Name(l) => format!("Name({:?})", l), 
            Expr::Temp(t) => format!("{:?}", t),       
            Expr::BinOp(op, l, r) => {
                format!(
                    "BinOp({:?},\n{},\n{})",
                    op,
                    space(indent + 1) + &l.pretty(indent + 1),
                    space(indent + 1) + &r.pretty(indent + 1)
                )
            }
            Expr::Mem(e) => format!("Mem({})", e.pretty(indent)),
            Expr::Call(fun, args) => {
                let mut s = format!("Call({},\n", fun.pretty(indent + 1));
                for (i, arg) in args.iter().enumerate() {
                    s.push_str(&space(indent + 1));
                    s.push_str(&arg.pretty(indent + 1));
                    if i < args.len() - 1 {
                        s.push_str(",\n");
                    }
                }
                s.push_str(")");
                s
            }
            Expr::ESeq(s, e) => {
                format!(
                    "ESeq(\n{},\n{}{})",
                    s.pretty(indent + 1),
                    space(indent + 1),
                    e.pretty(indent + 1)
                )
            }
        }
    }
}

impl Pretty for Stmt {
    fn pretty(&self, indent: usize) -> String {
        let s = space(indent);
        match self {
            Stmt::Move(dst, src) => {
                format!(
                    "{}Move(\n{}{},\n{}{}\n{})",
                    s,
                    space(indent + 1),
                    dst.pretty(indent + 1),
                    space(indent + 1),
                    src.pretty(indent + 1),
                    s
                )
            }
            Stmt::Expr(e) => format!("{}Expr({})", s, e.pretty(indent)),
            Stmt::Jump(e, _) => format!("{}Jump({})", s, e.pretty(indent)),
            Stmt::CJump(op, e1, e2, t, f) => {
                format!(
                    "{}CJump({:?}, {}, {}, {:?}, {:?})",
                    s,
                    op,
                    e1.pretty(0),
                    e2.pretty(0),
                    t,
                    f
                )
            }
            Stmt::Seq(l, r) => {
                format!("{}\n{}", l.pretty(indent), r.pretty(indent))
            }
            Stmt::Label(l) => format!("{}{:?}", s, l),
        }
    }
}

impl Pretty for Fragment {
    fn pretty(&self, indent: usize) -> String {
        match self {
            Fragment::Proc { label, body, frame } => {
                let s = space(indent);
                format!(
                    "{}Proc({:?}) {{\n{}  Frame: {:?}\n{}\n{}}}",
                    s,
                    label,
                    s,
                    frame, // Assumes Frame has a reasonable Debug impl
                    body.pretty(indent + 1),
                    s
                )
            }
        }
    }
}

impl Pretty for Program {
    fn pretty(&self, _indent: usize) -> String {
        let mut out = String::from("Program {\n");
        for frag in &self.fragments {
            out.push_str(&frag.pretty(1));
            out.push_str("\n\n"); // Double newline for spacing between functions
        }
        out.push_str("}");
        out
    }
}
