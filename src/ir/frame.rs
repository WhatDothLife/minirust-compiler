use crate::ir;
use crate::ir::translate;

use super::symbols::Temp;

/// Represents how a variable or sub-expression is accessed within a frame.
///
/// In this "Virtual Register" model, all local variables are initially 
/// treated as having an infinite supply of registers. 
#[derive(Clone, Debug)]
pub enum Access {
    /// The value resides in a virtual register (Temp). 
    ///
    /// During the "Pure Temp" IR phase, every variable is assigned a `Temp`.
    /// The "Lazy Allocator" will later map these to specific stack offsets 
    /// or physical registers during the Final Assembly Rewrite.
    InReg(Temp),

    // NOTE InFrame(i32) is currently omitted to support the "Pure Temp" approach.
    // Re-introduce only for "Escaping" variables if nested functions are added,
    // or for variables that must reside in memory (like arrays/structs).
}

pub const WORD_SIZE: i32 = 8; // for 64-bit architecture

#[derive(Clone, Debug)]
pub struct Frame {
    name: String,
    offset_counter: i32,
}

impl Frame {
    pub fn new(name: String) -> Self {
        Self {
            name,
            offset_counter: 0,
        }
    }

    pub fn alloc_local(&mut self) -> Access {
        // NOTE In the case of "Escaping" variables the code would look like
        // this:
        // self.offset_counter -= WORD_SIZE;
        // Access::InFrame(self.offset_counter)

        Access::InReg(Temp::new())
    }

    pub fn alloc_params(&mut self, count: usize) -> Vec<Access> {
        if count > 6 {
            panic!("Only up to 6 arguments are supported.");
        }

        (0..count).map(|_| self.alloc_local()).collect()
    }

    pub fn get_incoming_arg(&self, i: usize) -> translate::Expr {
        if i < Temp::ARG_REGS.len() {
            translate::Expr::Ex(ir::Expr::Temp(Temp::ARG_REGS[i]))
        } else {
            panic!("Only up to 6 arguments are supported in this implementation.");
        }
    }

}
