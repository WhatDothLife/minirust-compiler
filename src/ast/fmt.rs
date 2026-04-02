use std::fmt::Display;

use crate::util::Pretty;

use super::{BinOp, Expr, FunSignature, Program, Top, Type};

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "bool"),
            Type::I64 => write!(f, "i64"),
            Type::Fun(args, ret) => {
                let arg_list = args
                    .inner()
                    .iter()
                    .map(|ty_tag| format!("{}", ty_tag.inner())) 
                    .collect::<Vec<String>>()
                    .join(", ");

                write!(f, "fn({}) -> {}", arg_list, ret.inner())
            }
        }
    }
}

impl Pretty for BinOp {
    fn pretty(&self, _: usize) -> String {
        match self {
            BinOp::Add => "+".to_string(),
            BinOp::Sub => "-".to_string(),
            BinOp::Mul => "*".to_string(),
            BinOp::Div => "/".to_string(),
            BinOp::Gt => ">".to_string(),
            BinOp::Gte => ">=".to_string(),
            BinOp::Lt => "<".to_string(),
            BinOp::Lte => "<=".to_string(),
            BinOp::Eq => "==".to_string(),
            BinOp::Ne => "!=".to_string(),
        }
    }
}

impl Pretty for Type {
    fn pretty(&self, _: usize) -> String {
        format!("{}", self)
    }
}

fn space(n: usize) -> String {
    "  ".repeat(n)
}

impl Pretty for Expr {
    fn pretty(&self, indent: usize) -> String {
        let s = space(indent);
        let next = indent + 1;

        match self {
            Expr::Unit => "Unit".to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::Int(n) => format!("Int({})", n),
            Expr::Ident(id) => format!("Ident(\"{}\")", id.inner()),

            Expr::BinOp(l, op, r) => {
                // Explicitly unwrap tags for children to avoid the "Tag { ... }" clutter
                format!(
                    "BinOp({:?}, {}, {})",
                    op.inner(),
                    l.inner().pretty(0),
                    r.inner().pretty(0)
                )
            }

            Expr::FunDec(sig, continuation) => {
                format!(
                    "{}\n{}Next: {}",
                    sig.pretty(indent),
                    s,
                    continuation.inner().pretty(indent + 1)
                )
            }

            Expr::Let(name, ty, val, continuation) => {
                let ty_str = ty
                    .as_ref()
                    .map(|t| format!(": {}", t.inner()))
                    .unwrap_or_default();

                format!(
                    "Let(name: {}{}, val: {})\n{}Next: {}",
                    name.inner(),
                    ty_str,
                    val.inner().pretty(next), // Use 'next' instead of 0
                    s,
                    continuation.inner().pretty(next)
                )
            }

            Expr::Seq(e1, e2) => {
                format!(
                    "Seq(\n{}{},\n{}{}\n{})",
                    space(next),
                    e1.inner().pretty(next),
                    space(next),
                    e2.inner().pretty(next),
                    s
                )
            }

            Expr::If(cond, then, els) => {
                format!(
                    "If(\n{}cond: {},\n{}then: {},\n{}else: {}\n{})",
                    space(next),
                    cond.inner().pretty(0),
                    space(next),
                    then.inner().pretty(next),
                    space(next),
                    els.inner().pretty(next),
                    s
                )
            }

            Expr::FunApp(func, args) => {
                let args_str: Vec<String> =
                    args.inner().iter().map(|a| a.inner().pretty(0)).collect();
                format!(
                    "Call({}, [{}])",
                    func.inner().pretty(0),
                    args_str.join(", ")
                )
            }

            Expr::Print(e) => format!("Print({})", e.inner().pretty(0)),
        }
    }
}

impl Pretty for FunSignature {
    fn pretty(&self, indent: usize) -> String {
        let s = space(indent);
        let next = indent + 1;

        let params: Vec<String> = self
            .params
            .inner()
            .iter()
            .map(|(id, ty)| format!("{}: {}", id.inner(), ty.inner()))
            .collect();

        format!(
            "FunDec(\n{}name: {},\n{}args: [{}], ret: {},\n{}body: {}\n{})",
            space(next),
            self.name.inner(),
            space(next),
            params.join(", "),
            self.ret.inner(),
            space(next),
            self.body.inner().pretty(next),
            s
        )
    }
}

impl Pretty for Program {
    fn pretty(&self, indent: usize) -> String {
        let mut out = String::from("Program([\n");
        for top in &self.0 {
            match top.inner() {
                Top::FunDec(sig) => {
                    out.push_str(&space(indent + 1));
                    out.push_str(&sig.pretty(indent + 1));
                    out.push_str("\n");
                }
            }
        }
        out.push_str("])");
        out
    }
}
