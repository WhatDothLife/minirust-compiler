use crate::ir;
use crate::ir::translate;

use super::symbols::Temp;

#[derive(Clone, Debug)]
pub enum Access {
    InFrame(i32),
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
        self.offset_counter -= WORD_SIZE;
        Access::InFrame(self.offset_counter)
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

    // // Frame pointer
    // pub fn fp() -> Temp {
    //     Temp(0)
    // }

    // // Return Value register (e.g., %rax)
    // pub fn rv() -> Temp {
    //     Temp(1)
    // }

    // // Stack pointer
    // pub fn sp() -> Temp {
    //     Temp(2)
    // }
}
