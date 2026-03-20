use std::fmt::Display;

use super::Type;

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "()"),
            Type::Int => write!(f, "Int"),
            Type::Fun(args, ret) => write!(
                f,
                "({}) -> {}",
                args.iter()
                    .map(|ty| format!("{}", ty))
                    .collect::<Vec<String>>()
                    .join(","),
                ret
            ),
        }
    }
}

