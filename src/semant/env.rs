use std::collections::HashMap;

use crate::{
    ast::{Tag, Type},
    ir::{frame::Access, symbols::Label},
};

#[derive(Clone, Debug)]
pub enum Binding {
    /// A standard variable
    Var(Access),
    /// A function defined by a label
    Fun(Label),
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub ty: Tag<Type>,
    pub binding: Binding,
}

#[derive(Clone, Debug)]
pub struct Environment {
    globals: HashMap<String, Entry>,
    locals: HashMap<String, Entry>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            locals: HashMap::new(),
        }
    }

    pub fn insert_local(&mut self, name: String, ty: Entry) {
        self.locals.insert(name, ty);
    }

    pub fn insert_global(&mut self, name: String, entry: Entry) {
        self.globals.insert(name, entry);
    }

    /// Creates a fresh environment for a function body.
    /// It keeps the globals but wipes the locals.
    pub fn for_body(&self) -> Self {
        Self {
            globals: self.globals.clone(),
            locals: HashMap::new(),
        }
    }

    /// Used for Let-bindings and parameters
    pub fn with_local(&self, name: String, entry: Entry) -> Self {
        let mut clone = self.clone();
        clone.locals.insert(name, entry);
        clone
    }

    /// Used for blocks
    pub fn enter_scope(&self) -> Self {
        self.clone()
    }

    pub fn lookup(&self, name: &str) -> Option<&Entry> {
        self.locals.get(name).or_else(|| self.globals.get(name))
    }
}
