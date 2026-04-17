mod err;
mod fmt;
mod tag;

pub use err::Error;
pub use err::Result;
pub use tag::Tag;

pub type _BinOp = Tag<BinOp>;
pub type _Var = Tag<String>;
pub type _Expr = Box<Tag<Expr>>;
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
    Ne,
}

#[derive(Clone)]
pub enum Type {
    Unit,
    Bool,
    I64,
    Fun(_Vec<_Type>, Box<_Type>),
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "bool"),
            Type::I64 => write!(f, "i64"),
            Type::Fun(params, ret) => {
                write!(f, "fn(")?;
                for (i, param) in params.inner().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", param)?;
                }
                write!(f, ") -> {:?}", ret)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Unit,
    Bool(bool),
    Ident(_Var),
    Int(i64),
    BinOp(_Expr, _BinOp, _Expr),

    If(_Expr, _Expr, _Expr),
    FunApp(_Expr, _Vec<_Expr>),
    Print(_Expr),
    Block(_Expr),

    Let(_Ident, Option<_Type>, _Expr, _Expr),
    FunDec(FunSignature, _Expr),
    Seq(_Expr, _Expr),
}

#[derive(Clone, Debug)]
pub struct FunSignature {
    pub name: _Ident,
    pub params: _Vec<(_Ident, _Type)>,
    pub ret: _Type,
    pub body: _Expr,
}

#[derive(Clone, Debug)]
pub enum Top {
    FunDec(FunSignature),
}

pub type _Top = Tag<Top>;

pub type Program = Vec<_Top>;
