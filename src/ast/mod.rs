mod err;
mod fmt;
mod tag;

pub use tag::Tag;
pub use err::Error;
pub use err::Result;

pub type _BinOp = Tag<BinOp>;
pub type _Var = Tag<String>;
pub type _Expr = Tag<Expr>;
pub type _Int = Tag<i64>;
pub type _Vec<T> = Tag<Vec<T>>;
pub type _Ident = Tag<String>;
pub type _Type = Tag<Type>;

#[derive(Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,

    Gt,
    Gte,
    Lt,
    Lte,

    Eq,
    Neq,
}

#[derive(Clone, Debug)]
pub enum Type {
    Unit,
    Bool,
    Int,
    Fun(_Vec<_Type>, _Type),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Unit,
    Ident(_Var),
    Int(i64),
    BinOp(_Expr, _BinOp, _Expr),

    Seq(_Expr, _Expr),

    FunApp(_Expr, _Vec<_Expr>),
    Print(_Expr),

    If(_Expr, _Expr, _Expr),

    Let(_Ident, Option<_Type>, _Expr, _Expr),
    FunDec(_Ident, _Vec<(_Ident, _Type)>, _Type, _Expr, _Expr),
}

pub type _Top = Tag<Top>;
#[derive(Clone, Debug)]
pub enum Top {
    FunDec(_Ident, _Vec<(_Ident, _Type)>, _Type, _Expr),
}

pub type Program = Vec<_Top>;
