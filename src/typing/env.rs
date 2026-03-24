use std::collections::HashMap;

use crate::ast::Type;

#[derive(Clone, Debug)]
pub struct Environment(HashMap<String, Type>);

impl Environment {
    pub fn new() -> Self {
        Environment(HashMap::new())
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.0.insert(name, ty);
    }

    pub fn mutate(&mut self, name: String, ty: Type) -> Self {
        let mut clone = self.clone();
        clone.insert(name, ty);
        clone
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.0.get(name)
    }
}
