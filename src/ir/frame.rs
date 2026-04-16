use std::collections::HashMap;

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
    temp_map: HashMap<Temp, i32>,
}

impl Frame {
    pub fn new(name: String) -> Self {
        Self {
            name,
            offset_counter: -24,
            temp_map: HashMap::new(),
        }
    }

    pub fn size(&self) -> i32 {
        let total_raw_size = self.offset_counter.abs();

        let alignment = 16;
        (total_raw_size + (alignment - 1)) & !(alignment - 1)
    }

    pub fn alloc_local(&mut self) -> Access {
        let new_temp = Temp::new();

        let off = self.offset_counter;
        self.temp_map.insert(new_temp, off);

        self.offset_counter -= WORD_SIZE;

        Access::InReg(new_temp)
    }

    pub fn alloc_params(&mut self, count: usize) -> Vec<Access> {
        if count > 8 {
            panic!("Only up to 8 arguments are supported.");
        }

        (0..count).map(|_| self.alloc_local()).collect()
    }

    pub fn get_incoming_arg(&self, i: usize) -> translate::Expr {
        if i < Temp::ARG_REGS.len() {
            translate::Expr::Ex(ir::Expr::Temp(Temp::ARG_REGS[i]))
        } else {
            panic!("Only up to 8 arguments are supported in this implementation.");
        }
    }

    pub fn get_offset(&mut self, t: Temp) -> i32 {
        *self.temp_map.entry(t).or_insert_with(|| {
            let off = self.offset_counter;
            self.offset_counter -= 8; // Move down 8 bytes for next temp
            off
        })
    }
}
