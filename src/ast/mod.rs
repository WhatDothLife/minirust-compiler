pub mod err;
mod fmt;
mod tag;

pub use tag::Tag;

pub type _BinOp = Tag<BinOp>;
pub type _Var = Tag<String>;
pub type _Term = Tag<Term>;
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
    Int,
    Fun(_Vec<_Type>, _Type),
}

#[derive(Clone, Debug)]
pub enum Term {
    Unit,
    Var(_Var),
    Int(i64),
    BinOp(_Term, _BinOp, _Term),

    Seq(_Term, _Term),

    FunApp(_Term, _Vec<_Term>),

    Let(_Ident, Option<_Type>, _Term, _Term),
    FunDec(_Ident, _Vec<(_Ident, _Type)>, _Type, _Term, _Term),
}

pub type _Top = Tag<Top>;
#[derive(Clone, Debug)]
pub enum Top {
    FunDec(_Ident, _Vec<(_Ident, _Type)>, _Type, _Term),
}

pub type Program = Vec<_Top>;
