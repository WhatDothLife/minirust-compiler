mod label;
mod temp;
pub mod translate;
mod tree;

use label::Label;
pub use tree::Expr;
use tree::Stmt;

pub struct IrFunction {
    pub name: Label,
    pub body: Stmt,
}

pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}
