use std::collections::HashMap;

use crate::ast::Type;

pub struct Environment(HashMap<String, EnvEntry>);

enum EnvEntry {
    VarEntry(Type),
    FunEntry { formals: Vec<Type>, result: Type },
}
