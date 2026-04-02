use crate::util::Boxed;

use super::{Expr, Stmt, Temp};

/// Checks if a statement and an expression commute.
///
/// Commutativity exists if the execution of the statement does not affect
/// the value of the expression, and the evaluation of the expression
/// does not affect the side effects of the statement.
///
/// If this returns true, the compiler is free to reorder these operations
/// without changing the program's semantics. If false, the statement
/// must be executed before the expression, or the expression must be
/// preserved (e.g., in a temporary register).
fn commutes(stmt: &Stmt, expr: &Expr) -> bool {
    match (stmt, expr) {
        (Stmt::Expr(Expr::Const(_)), _) => true,
        (_, Expr::Name(_) | Expr::Const(_)) => true,
        _ => false,
    }
}

/// The core reordering logic: Takes a list of expressions and extracts
/// all nested `ESeq` nodes to hoist their side effects.
///
/// This function:
/// 1. Recursively traverses the expression list to identify and extract
///    nested side effects (represented as `ESeq`s).
/// 2. Ensures that all side effects are moved into a single, sequential
///    `Stmt` that precedes the evaluation of the cleaned expressions.
/// 3. Maintains the original program semantics by verifying commutativity
///    between statements and expressions.
///
/// Returns a tuple containing:
/// - A `Stmt` comprising all extracted side effects (to be executed first).
/// - A `Vec<Expr>` of "clean" expressions (free of nested `ESeq`s).
fn reorder(mut exprs: Vec<Expr>) -> (Stmt, Vec<Expr>) {
    if exprs.is_empty() {
        return (Stmt::nop(), vec![]);
    }

    if let Some(Expr::Call(_, _)) = exprs.get(0) {
        let t = Temp::new();
        let call = exprs.remove(0);
        let eseq = Expr::ESeq(
            Stmt::Move(Expr::Temp(t), call).boxed(),
            Expr::Temp(t).boxed(),
        );
        exprs.insert(0, eseq);
        return reorder(exprs);
    }

    let (stmts, clean_expr) = canon_expr(exprs.remove(0));
    let (stmts_rest, mut clean_exprs) = reorder(exprs);

    if commutes(&stmts_rest, &clean_expr) {
        clean_exprs.insert(0, clean_expr);
        (Stmt::Seq(stmts.boxed(), stmts_rest.boxed()), clean_exprs)
    } else {
        let t = Temp::new();
        let move_to_temp = Stmt::Move(Expr::Temp(t), clean_expr);
        clean_exprs.insert(0, Expr::Temp(t));
        (
            Stmt::Seq(
                stmts.boxed(),
                Stmt::Seq(move_to_temp.boxed(), stmts_rest.boxed()).boxed(),
            ),
            clean_exprs,
        )
    }
}

/// Helper function that reorders a list of expressions associated with a statement
/// by hoisting their side effects.
///
/// This function:
/// 1. Processes each `Expr` in the list to extract nested `ESeq` nodes.
/// 2. Manages the execution order of side effects, ensuring they occur before
///    the main statement logic.
/// 3. Uses the provided `assemble` closure to reconstruct the statement
///    (e.g., `Jump`, `CJump`, or `Move`) using the cleaned, side-effect-free expressions.
///
/// The returned `Stmt` encapsulates both the extracted side effects and
/// the reconstructed statement, ensuring that the original semantics are preserved.
pub fn reorder_stmt_with<F>(exprs: Vec<Expr>, assemble: F) -> Stmt
where
    F: FnOnce(Vec<Expr>) -> Stmt,
{
    let (stms, clean_exprs) = reorder(exprs);
    Stmt::Seq(stms.boxed(), assemble(clean_exprs).boxed())
}

/// Helper function that reorders a list of expressions by hoisting side effects
/// (represented as `ESeq` nodes) into a separate `Stmt`.
///
/// This function:
/// 1. Processes each `Expr` in the list to extract nested side effects.
/// 2. Ensures proper evaluation order (left-to-right) and commutativity.
/// 3. Uses the provided `assemble` closure to reconstruct the expression tree
///    from the cleaned, side-effect-free expressions.
///
/// Returns a tuple of:
/// - A `Stmt` containing the extracted side effects (to be executed first).
/// - The reconstructed `Expr` (now free of nested `ESeq`s).
pub fn reorder_expr_with<F>(exprs: Vec<Expr>, assemble: F) -> (Stmt, Expr)
where
    F: FnOnce(Vec<Expr>) -> Expr,
{
    let (stms, clean_exprs) = reorder(exprs);
    (stms, assemble(clean_exprs))
}

/// Transforms a statement into a canonical "flat" form by removing all nested `ESeq` nodes.
///
/// This function recursively descends into the statement tree, extracting any
/// side-effect-heavy expressions (like those containing `ESeq`) and converting
/// them into a sequence of simple, sequential statements.
///
/// The resulting statement is guaranteed to be free of nested `ESeq` expressions,
/// making it ready for instruction selection and code generation.
fn canon_stmt(stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Seq(s1, s2) => Stmt::Seq(canon_stmt(*s1).boxed(), canon_stmt(*s2).boxed()),
        Stmt::Jump(e, labels) => {
            reorder_stmt_with(vec![e], |mut el| Stmt::Jump(el.remove(0), labels))
        }
        Stmt::CJump(op, e1, e2, t, f) => reorder_stmt_with(vec![e1, e2], |mut el| {
            let e2 = el.pop().unwrap();
            let e1 = el.pop().unwrap();
            Stmt::CJump(op, e1, e2, t, f)
        }),
        Stmt::Move(Expr::Temp(t), Expr::Call(f, args)) => {
            let mut exprs = vec![*f];
            exprs.extend(args);
            reorder_stmt_with(exprs, |mut el| {
                let f_clean = el.remove(0);
                Stmt::Move(Expr::Temp(t), Expr::Call(f_clean.boxed(), el))
            })
        }
        Stmt::Move(Expr::Temp(t), src) => {
            reorder_stmt_with(vec![src], |mut el| Stmt::Move(Expr::Temp(t), el.remove(0)))
        }
        Stmt::Move(Expr::Mem(addr), src) => reorder_stmt_with(vec![*addr, src], |mut el| {
            let src_clean = el.pop().unwrap();
            let addr_clean = el.pop().unwrap();
            Stmt::Move(Expr::Mem(addr_clean.boxed()), src_clean)
        }),
        Stmt::Expr(e) => reorder_stmt_with(vec![e], |mut el| Stmt::Expr(el.remove(0))),
        other => other,
    }
}

/// Transforms an expression into a canonical form:
/// Returns a tuple containing:
/// 1. A `Stmt` representing all side effects that must be executed before evaluating the expression.
/// 2. A "clean" `Expr` that is free of any `ESeq` nodes and is safe to use in
///    contexts where evaluation order matters.
///
/// This process effectively "hoists" side effects out of expression trees
/// and ensures that evaluation order is strictly preserved.
fn canon_expr(expr: Expr) -> (Stmt, Expr) {
    match expr {
        Expr::BinOp(op, l, r) => reorder_expr_with(vec![*l, *r], |mut clean_exprs| {
            let r_clean = clean_exprs.pop().unwrap();
            let l_clean = clean_exprs.pop().unwrap();
            Expr::BinOp(op, l_clean.boxed(), r_clean.boxed())
        }),
        Expr::Mem(addr) => reorder_expr_with(vec![*addr], |mut el| Expr::Mem(el.remove(0).boxed())),
        Expr::ESeq(s, e) => {
            let stms1 = canon_stmt(*s);
            let (stms2, e_clean) = canon_expr(*e);
            (Stmt::Seq(stms1.boxed(), stms2.boxed()), e_clean)
        }
        Expr::Call(f, args) => {
            let mut exprs = vec![*f];
            exprs.extend(args);
            reorder_expr_with(exprs, |mut el| {
                let f_clean = el.remove(0);
                Expr::Call(f_clean.boxed(), el)
            })
        }
        other => (Stmt::nop(), other), // Const, Temp, Name
    }
}

pub fn linearize(stmt: Stmt) -> Vec<Stmt> {
    let mut instructions = Vec::new();

    fn flatten(stmt: Stmt, acc: &mut Vec<Stmt>) {
        match stmt {
            Stmt::Seq(a, b) => {
                flatten(*a, acc);
                flatten(*b, acc);
            }

            Stmt::Expr(Expr::Const(0)) => {} // nop
            s => acc.push(s),
        }
    }

    flatten(canon_stmt(stmt), &mut instructions);

    instructions
}
