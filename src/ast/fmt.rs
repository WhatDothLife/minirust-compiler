use std::fmt::Display;

use super::Type;

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "i64"),
            Type::Fun(args, ret) => write!(
                f,
                "({}) -> {}",
                args.inner().iter()
                    .map(|ty| format!("{}", ty))
                    .collect::<Vec<String>>()
                    .join(","),
                ret
            ),
        }
    }
}

