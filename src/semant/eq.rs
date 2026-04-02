use crate::ast::{Error, Result, Type, _Type, _Vec};

impl Type {
    // Checks whether two types are structurally identical.
    pub fn matches(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Unit, Type::Unit) | (Type::Bool, Type::Bool) | (Type::I64, Type::I64) => true,
            (Type::Fun(l_args, l_ret), Type::Fun(r_args, r_ret)) => {
                l_args.inner().len() == r_args.inner().len()
                    && l_args
                        .inner()
                        .iter()
                        .zip(r_args.inner())
                        .all(|(l, r)| l.inner().matches(r.inner()))
                    && l_ret.inner().matches(r_ret.inner())
            }
            _ => false,
        }
    }
}

impl _Type {
    pub fn assert_eq(&self, expected: &_Type) -> Result<()> {
        match (self.inner(), expected.inner()) {
            (Type::Fun(l_args, l_ret), Type::Fun(r_args, r_ret)) => {
                len_eq(l_args, r_args)?;
                for (found_arg, exp_arg) in l_args.inner().iter().zip(r_args.inner()) {
                    found_arg.assert_eq(exp_arg)?;
                }
                l_ret.assert_eq(r_ret)
            }

            _ if self.inner().matches(expected.inner()) => Ok(()),

            _ => Err(Error::new("type mismatch")
                .label(format!("expected {}", expected.inner()), expected.span())
                .label(format!("found {}", self.inner()), self.span())),
        }
    }

    pub fn assert_eq_raw(&self, expected: &Type) -> Result<()> {
        if self.inner().matches(expected) {
            Ok(())
        } else {
            Err(Error::new("type mismatch").label(format!("expected {}", expected), self.span()))
        }
    }
}

pub fn len_eq<A, B>(found: &_Vec<A>, exp: &_Vec<B>) -> Result<()> {
    if found.inner().len() != exp.inner().len() {
        return Err(Error::new("count mismatch")
            .label(
                format!("expected {} arguments", exp.inner().len()),
                exp.span(),
            )
            .label(format!("found {}", found.inner().len()), found.span()));
    }
    Ok(())
}
