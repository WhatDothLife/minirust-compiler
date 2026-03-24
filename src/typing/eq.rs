use crate::ast::{Error, Result, Type, _Type, _Vec};

pub trait TypeEq {
    fn assert_eq(&self, rhs: &Self) -> Result<()>;
}

impl TypeEq for _Type {
    fn assert_eq(&self, rhs: &_Type) -> Result<()> {
        match (self.inner(), rhs.inner()) {
            (Type::Unit, Type::Unit) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),
            (Type::Int, Type::Int) => Ok(()),

            (Type::Fun(l_args, l_ret), Type::Fun(r_args, r_ret)) => {
                l_args.assert_eq(r_args)?;
                l_ret.assert_eq(r_ret)
            }

            (_, _) => Err(Error::new("type mismatch")
                .label(format!("expected {}", rhs.inner()), rhs.span())
                .label(format!("found {}", self.inner()), self.span())),
        }
    }
}

pub fn len_eq<A, B>(left: &_Vec<A>, right: &_Vec<B>) -> Result<()> {
    let l_len = left.inner().len();
    let r_len = right.inner().len();

    if l_len != r_len {
        return Err(Error::new("count mismatch")
            .label(format!("expected {}", r_len), right.span())
            .label(format!("found {}", l_len), left.span()));
    }
    Ok(())
}

impl TypeEq for _Vec<_Type> {
    fn assert_eq(&self, rhs: &_Vec<_Type>) -> Result<()> {
        len_eq(self, rhs)?;

        for (left, right) in self.inner().iter().zip(rhs.inner()) {
            left.assert_eq(right)?;
        }
        Ok(())
    }
}
