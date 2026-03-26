use super::{label::Label, temp::Temp, tree};

pub enum Expr {
    Ex(tree::Expr),
    Nx(tree::Stmt),
    Cx(Box<dyn Fn(Label, Label) -> tree::Stmt>),
}

pub fn un_ex(expr: Expr) -> tree::Expr {
    match expr {
        Expr::Ex(expr) => expr,
        // Use dummy constant because no value
        Expr::Nx(stmt) => tree::Expr::ESeq(Box::new(stmt), Box::new(tree::Expr::Const(0))),
        Expr::Cx(gen_stmt) => {
            let r = Temp::new();
            let t = Label::new();
            let f = Label::new();

            let stmts = tree::seq([
                tree::Stmt::Move(tree::Expr::Temp(r), tree::Expr::Const(1)),
                gen_stmt(t.clone(), f.clone()),
                tree::Stmt::Label(f),
                tree::Stmt::Move(tree::Expr::Temp(r), tree::Expr::Const(0)),
                tree::Stmt::Label(t),
            ])
            .unwrap();

            tree::Expr::ESeq(Box::new(stmts), Box::new(tree::Expr::Temp(r)))
        }
    }
}

pub fn un_nx(expr: Expr) -> tree::Stmt {
    match expr {
        Expr::Ex(expr) => tree::Stmt::Expr(expr),
        Expr::Nx(stmt) => stmt,
        Expr::Cx(gen_stmt) => {
            let join = Label::new();
            let statement = gen_stmt(join.clone(), join.clone());
            tree::Stmt::Seq(Box::new(statement), Box::new(tree::Stmt::Label(join)))
        }
    }
}

pub fn un_cx(expr: Expr) -> Box<dyn Fn(Label, Label) -> tree::Stmt> {
    match expr {
        Expr::Ex(e) => Box::new(move |t, f| {
            tree::Stmt::CJump(tree::RelOp::Ne, e.clone(), tree::Expr::Const(0), t, f)
        }),
        Expr::Cx(gen_stmt) => gen_stmt,
        Expr::Nx(_) => unreachable!(),
    }
}

pub fn new_if_exp(cond: Expr, then_b: Expr, else_b: Expr) -> Expr {
    let gen_cond = un_cx(cond);
    
    let t = Label::new();
    let f = Label::new();
    let join = Label::new();
    let r = Temp::new();

    let stmts = tree::seq([
        gen_cond(t.clone(), f.clone()),
        tree::Stmt::Label(t),
        tree::Stmt::Move(tree::Expr::Temp(r), un_ex(then_b)),
        tree::Stmt::Jump(tree::Expr::Name(join.clone()), vec![join.clone()]),
        tree::Stmt::Label(f),
        tree::Stmt::Move(tree::Expr::Temp(r), un_ex(else_b)),
        tree::Stmt::Label(join),
    ]).unwrap();

    Expr::Ex(tree::Expr::ESeq(Box::new(stmts), Box::new(tree::Expr::Temp(r))))
}
