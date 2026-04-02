use frame::Frame;
use symbols::{Label, Temp};

use crate::util::Boxed;

mod fmt;
pub mod canonical;

pub mod frame;
pub mod symbols;
pub mod translate;

#[derive(Clone, Debug)]
pub enum Expr {
    /// Integer constant
    Const(i64),

    /// Corresponds to an assembly-language label (symbolic address)
    Name(Label),

    /// Temporary value (similar to a register in a real machine)
    Temp(Temp),

    /// Binary operation: op(left, right)
    BinOp(BinOp, Box<Expr>, Box<Expr>),

    /// Memory access:
    /// Contents of `wordSize` bytes of memory starting at address `e`
    /// (wordSize is defined by the frame module)
    Mem(Box<Expr>),

    /// Function call:
    /// `fun(args...)`
    /// The function expression is evaluated before the arguments.
    Call(Box<Expr>, Vec<Expr>),

    /// ESeq(stmt, expr):
    /// Execute `stmt` first, then evaluate `expr` and return its value.
    /// Used to enforce evaluation order.
    ESeq(Box<Stmt>, Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    /// Move(dst, src)
    ///
    /// Move(Temp t, e):
    ///     Evaluate `e` and store the result in temporary `t`.
    ///
    /// Move(Mem(e1), e2):
    ///     Evaluate `e1`, yielding address `a`.
    ///     Then evaluate `e2` and store the result into memory at address `a`
    ///     (word size determined by frame module).
    Move(Expr, Expr),

    /// Expr(e)
    ///
    /// Evaluate `e` and discard the result.
    /// Used for expressions with side effects (e.g. function calls).
    Expr(Expr),

    /// Jump(e, labels)
    ///
    /// Transfer control to address `e`.
    /// The destination `e` may be a literal label `Name(lab)`
    /// or a computed address.
    ///
    /// The label list contains all possible jump targets.
    /// This is required for later data-flow analysis.
    ///
    /// Common case:
    ///     Jump(Name(l), [l])
    Jump(Expr, Vec<Label>),

    /// CJump(op, e1, e2, t, f)
    ///
    /// Conditional jump:
    /// Evaluate `e1` and `e2`, compare using `op`.
    ///
    /// If true  -> jump to label `t`
    /// If false -> jump to label `f`
    CJump(RelOp, Expr, Expr, Label, Label),

    /// Sequential execution:
    /// First execute left statement, then right statement.
    Seq(Box<Stmt>, Box<Stmt>),

    /// Label(l)
    ///
    /// Define the constant value of label `l` to be the current
    /// machine-code address.
    ///
    /// Equivalent to a label definition in assembly.
    /// Expressions like `Name(l)` may be used as jump targets.
    Label(Label),
}

impl Stmt {
    fn nop() -> Stmt {
        Stmt::Expr(Expr::Const(0))
    }
}

impl Boxed for Expr {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Boxed for Stmt {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Shl,
    Shr,
    Ashr,
    Xor,
}

#[derive(Clone, Copy, Debug)]
pub enum RelOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Ult,
    Ule,
    Ugt,
    Uge,
}

#[derive(Clone, Debug)]
pub enum Fragment {
    Proc {
        label: Label,
        body: Stmt,
        frame: Frame,
    },
}

#[derive(Clone, Debug)]
pub struct Program {
    pub fragments: Vec<Fragment>,
}
