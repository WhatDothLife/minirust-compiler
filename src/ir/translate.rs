use super::{
    frame::{Access, Frame},
    symbols::{Label, Temp},
    Fragment,
};

use crate::{ir, util::Boxed};

pub enum Expr {
    Ex(ir::Expr),
    Nx(ir::Stmt),
    Cx(Box<dyn FnOnce(Label, Label) -> ir::Stmt>),
}

fn un_ex(expr: Expr) -> ir::Expr {
    match expr {
        Expr::Ex(expr) => expr,
        // Use dummy constant because no value
        Expr::Nx(stmt) => ir::Expr::ESeq(stmt.boxed(), ir::Expr::Const(0).boxed()),
        Expr::Cx(gen_stmt) => {
            let r = Temp::new();
            let t = Label::new();
            let f = Label::new();

            let stmts = fold_stmts([
                ir::Stmt::Move(ir::Expr::Temp(r), ir::Expr::Const(1)),
                gen_stmt(t.clone(), f.clone()),
                ir::Stmt::Label(f),
                ir::Stmt::Move(ir::Expr::Temp(r), ir::Expr::Const(0)),
                ir::Stmt::Label(t),
            ])
            .unwrap();

            ir::Expr::ESeq(stmts.boxed(), ir::Expr::Temp(r).boxed())
        }
    }
}

fn un_nx(expr: Expr) -> ir::Stmt {
    match expr {
        Expr::Ex(expr) => ir::Stmt::Expr(expr),
        Expr::Nx(stmt) => stmt,
        Expr::Cx(gen_stmt) => {
            let join = Label::new();
            let stmt = gen_stmt(join.clone(), join.clone());
            ir::Stmt::Seq(stmt.boxed(), ir::Stmt::Label(join).boxed())
        }
    }
}

fn un_cx(expr: Expr) -> Box<dyn FnOnce(Label, Label) -> ir::Stmt> {
    match expr {
        Expr::Ex(e) => Box::new(move |t, f| {
            ir::Stmt::CJump(ir::RelOp::Ne, e.clone(), ir::Expr::Const(0), t, f)
        }),
        Expr::Cx(gen_stmt) => gen_stmt,
        Expr::Nx(_) => unreachable!(),
    }
}

fn fold_stmts<I: IntoIterator<Item = ir::Stmt>>(stmts: I) -> Option<ir::Stmt> {
    let mut iter = stmts.into_iter();
    let first = iter.next()?;
    let stmt = iter.fold(first, |acc, s| ir::Stmt::Seq(acc.boxed(), s.boxed()));

    Some(stmt)
}

/// Sequences a statement-like expression and a value-producing expression.
pub fn chain_expr(front: Expr, back: Expr) -> Expr {
    let stmt = un_nx(front);
    let expr = un_ex(back);

    Expr::Ex(ir::Expr::ESeq(stmt.boxed(), expr.boxed()))
}

// TODO Cases in which then_b or else_b are Cx should be recognized specially
pub fn if_else(cond: Expr, then_b: Expr, else_b: Expr) -> Expr {
    let gen_cond = un_cx(cond);

    let t = Label::new();
    let e = Label::new();
    let join = Label::new();
    let r = Temp::new();

    let stmts = fold_stmts([
        gen_cond(t.clone(), e.clone()),
        ir::Stmt::Label(t),
        ir::Stmt::Move(ir::Expr::Temp(r), un_ex(then_b)),
        ir::Stmt::Jump(ir::Expr::Name(join.clone()), vec![join.clone()]),
        ir::Stmt::Label(e),
        ir::Stmt::Move(ir::Expr::Temp(r), un_ex(else_b)),
        ir::Stmt::Label(join),
    ])
    .unwrap();

    Expr::Ex(ir::Expr::ESeq(stmts.boxed(), ir::Expr::Temp(r).boxed()))
}

pub fn unit() -> Expr {
    Expr::Ex(ir::Expr::Const(0))
}

pub fn int(i: i64) -> Expr {
    Expr::Ex(ir::Expr::Const(i))
}

fn mem(access: &Access) -> ir::Expr {
    match access {
        // Access::InFrame(offset) => ir::Expr::Mem(
        //     ir::Expr::BinOp(
        //         ir::BinOp::Add,
        //         ir::Expr::Temp(Temp::FP).boxed(),
        //         ir::Expr::Const(*offset as i64).boxed(),
        //     )
        //     .boxed(),
        // ), 
        Access::InReg(temp) => ir::Expr::Temp(*temp),
    }
}

pub fn var(access: &Access) -> Expr {
    Expr::Ex(mem(access))
}

pub fn fun(label: &Label) -> Expr {
    Expr::Ex(ir::Expr::Name(label.to_owned()))
}

fn arithmetic_op(op: ir::BinOp, l: Expr, r: Expr) -> Expr {
    Expr::Ex(ir::Expr::BinOp(op, un_ex(l).boxed(), un_ex(r).boxed()))
}

pub fn op_add(l: Expr, r: Expr) -> Expr {
    arithmetic_op(ir::BinOp::Add, l, r)
}

pub fn op_sub(l: Expr, r: Expr) -> Expr {
    arithmetic_op(ir::BinOp::Sub, l, r)
}

pub fn op_mul(l: Expr, r: Expr) -> Expr {
    arithmetic_op(ir::BinOp::Mul, l, r)
}

pub fn op_div(l: Expr, r: Expr) -> Expr {
    arithmetic_op(ir::BinOp::Div, l, r)
}

// Comparison

fn relational_op(op: ir::RelOp, l: Expr, r: Expr) -> Expr {
    let f = move |t, f| ir::Stmt::CJump(op, un_ex(l), un_ex(r), t, f);
    Expr::Cx(Box::new(f))
}

pub fn op_gt(l: Expr, r: Expr) -> Expr {
    relational_op(ir::RelOp::Gt, l, r)
}

pub fn op_ge(l: Expr, r: Expr) -> Expr {
    relational_op(ir::RelOp::Ge, l, r)
}

pub fn op_lt(l: Expr, r: Expr) -> Expr {
    relational_op(ir::RelOp::Lt, l, r)
}

pub fn op_le(l: Expr, r: Expr) -> Expr {
    relational_op(ir::RelOp::Le, l, r)
}

pub fn op_eq(l: Expr, r: Expr) -> Expr {
    relational_op(ir::RelOp::Eq, l, r)
}

pub fn op_ne(l: Expr, r: Expr) -> Expr {
    relational_op(ir::RelOp::Ne, l, r)
}

pub fn declare_var(val: Expr, cur_frame: &mut Frame) -> (Expr, Access) {
    let access = cur_frame.alloc_local();
    let assignment = ir::Stmt::Move(mem(&access), un_ex(val));

    (Expr::Nx(assignment), access)
}
pub fn get_label(id: &str) -> Label {
    Label::with_name(id)
}

pub fn print_int(e: Expr) -> Expr {
    let print_label = Label::with_name("print_int64");

    Expr::Ex(ir::Expr::Call(
        ir::Expr::Name(print_label).boxed(),
        vec![un_ex(e)],
    ))
}

pub fn fun_app(fun: Expr, args: Vec<Expr>) -> Expr {
    let ir_args = args.into_iter().map(un_ex).collect();

    Expr::Ex(ir::Expr::Call(un_ex(fun).boxed(), ir_args))
}

pub struct FunContext {
    pub label: Label,
    pub frame: Frame,
    pub params: Vec<(String, Access)>,
    // Reads out the values of the argument registers, if the function has no
    // arguments, the value is None.
    prolog: Option<ir::Stmt>,
}

impl FunContext {
    pub fn entry(label: Label, param_names: Vec<String>) -> Self {
        let mut frame = Frame::new(label.0.clone());
        let accesses = frame.alloc_params(param_names.len());

        let mut prolog_stmts = Vec::new();
        for (i, access) in accesses.iter().enumerate() {
            let dst = mem(access);
            let src = frame.get_incoming_arg(i);
            prolog_stmts.push(ir::Stmt::Move(dst, un_ex(src)));
        }

        let params = param_names.into_iter().zip(accesses).collect();
        let prolog = fold_stmts(prolog_stmts);

        Self {
            label,
            frame,
            params,
            prolog,
        }
    }

    pub fn exit(self, body_ir: Expr) -> Fragment {
        let body = un_ex(body_ir);
        let full_body = match self.prolog {
            Some(p) => ir::Expr::ESeq(p.boxed(), body.boxed()),
            None => body,
        };

        let exit_node = ir::Stmt::Move(ir::Expr::Temp(Temp::RV), full_body);
        let linearized = ir::canonical::linearize(exit_node); // Could be made more explicit instead of burying it here

        Fragment::Proc {
            label: self.label,
            body: linearized,
            frame: self.frame,
        }
    }
}
