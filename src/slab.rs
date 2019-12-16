use crate::error::Error;
use crate::parser::{ExpressionI, ValueI,
                    Expression,  Value};
use crate::compiler::{Instruction::{self, IConst}, InstructionI};

use std::fmt;
use std::mem;

#[cfg(feature="unsafe-vars")]
use std::collections::BTreeMap;

impl ExpressionI {
    #[inline]
    pub fn from(self, ps:&ParseSlab) -> &Expression {
        ps.get_expr(self)
    }
}
impl ValueI {
    #[inline]
    pub fn from(self, ps:&ParseSlab) -> &Value {
        ps.get_val(self)
    }
}

pub struct Slab {
    pub ps:ParseSlab,
    pub cs:CompileSlab,
}
pub struct ParseSlab {
               exprs      :Vec<Expression>,
               vals       :Vec<Value>,
               def_expr   :Expression,
               def_val    :Value,
    pub        char_buf   :String,
    #[cfg(feature="unsafe-vars")]
    pub(crate) unsafe_vars:BTreeMap<String, *const f64>,
}
pub struct CompileSlab {
    instrs   :Vec<Instruction>,
    def_instr:Instruction,
}
impl ParseSlab {
    #[inline]
    pub fn get_expr(&self, expr_i:ExpressionI) -> &Expression {
        // I'm using this non-panic match structure to boost performance:
        match self.exprs.get(expr_i.0) {
            Some(expr_ref) => expr_ref,
            None => &self.def_expr,
        }
    }
    #[inline]
    pub fn get_val(&self, val_i:ValueI) -> &Value {
        match self.vals.get(val_i.0) {
            Some(val_ref) => val_ref,
            None => &self.def_val,
        }
    }
    #[inline]
    pub(crate) fn push_expr(&mut self, expr:Expression) -> Result<ExpressionI,Error> {
        let i = self.exprs.len();
        if i>=self.exprs.capacity() { return Err(Error::SlabOverflow); }
        self.exprs.push(expr);
        Ok(ExpressionI(i))
    }
    #[inline]
    pub(crate) fn push_val(&mut self, val:Value) -> Result<ValueI,Error> {
        let i = self.vals.len();
        if i>=self.vals.capacity() { return Err(Error::SlabOverflow); }
        self.vals.push(val);
        Ok(ValueI(i))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.exprs.clear();
        self.vals.clear();
    }

    // TODO: Add "# Safety" section to docs.
    #[cfg(feature="unsafe-vars")]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn add_unsafe_var(&mut self, name:String, ptr:&f64) {
        self.unsafe_vars.insert(name, ptr as *const f64);
    }
}
impl CompileSlab {
    #[inline]
    pub fn get_instr(&self, i:InstructionI) -> &Instruction {
        match self.instrs.get(i.0) {
            Some(instr_ref) => instr_ref,
            None => &self.def_instr,
        }
    }
    pub(crate) fn push_instr(&mut self, instr:Instruction) -> InstructionI {
        if self.instrs.capacity()==0 { self.instrs.reserve(32); }
        let i = self.instrs.len();
        self.instrs.push(instr);
        InstructionI(i)
    }
    pub(crate) fn take_instr(&mut self, i:InstructionI) -> Instruction {
        if i.0==self.instrs.len()-1 {
            match self.instrs.pop() {
                Some(instr) => instr,
                None => IConst(std::f64::NAN),
            }
        } else {
            match self.instrs.get_mut(i.0) {
                Some(instr_ref) => mem::replace(instr_ref, IConst(std::f64::NAN)),  // Conspicuous Value
                None => IConst(std::f64::NAN),
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.instrs.clear();
    }
}
impl Slab {
    #[inline]
    pub fn new() -> Self { Self::with_capacity(64) }
    #[inline]
    pub fn with_capacity(cap:usize) -> Self {
        Self{
            ps:ParseSlab{
                exprs      :Vec::with_capacity(cap),
                vals       :Vec::with_capacity(cap),
                def_expr   :Default::default(),
                def_val    :Default::default(),
                char_buf   :String::with_capacity(64),
                #[cfg(feature="unsafe-vars")]
                unsafe_vars:BTreeMap::new(),
            },
            cs:CompileSlab{
                instrs   :Vec::new(),  // Don't pre-allocate for compilation.
                def_instr:Default::default(),
            },
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.ps.exprs.clear();
        self.ps.vals.clear();
        self.cs.instrs.clear();
    }
}


fn write_indexed_list<T>(f:&mut fmt::Formatter, lst:&[T]) -> Result<(), fmt::Error> where T:fmt::Debug {
    write!(f, "{{")?;
    let mut nonempty = false;
    for (i,x) in lst.iter().enumerate() {
        if nonempty { write!(f, ",")?; }
        nonempty = true;
        write!(f, " {}:{:?}",i,x)?;
    }
    if nonempty { write!(f, " ")?; }
    write!(f, "}}")?;
    Ok(())
}
impl fmt::Debug for Slab {
    fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Slab{{ exprs:")?;
        write_indexed_list(f, &self.ps.exprs)?;
        write!(f, ", vals:")?;
        write_indexed_list(f, &self.ps.vals)?;
        write!(f, ", instrs:")?;
        write_indexed_list(f, &self.cs.instrs)?;
        write!(f, " }}")?;
        Ok(())
    }
}
impl fmt::Debug for ParseSlab {
    fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ParseSlab{{ exprs:")?;
        write_indexed_list(f, &self.exprs)?;
        write!(f, ", vals:")?;
        write_indexed_list(f, &self.vals)?;
        write!(f, " }}")?;
        Ok(())
    }
}
impl fmt::Debug for CompileSlab {
    fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "CompileSlab{{ instrs:")?;
        write_indexed_list(f, &self.instrs)?;
        write!(f, " }}")?;
        Ok(())
    }
}

impl Default for Slab {
    fn default() -> Self { Self::new() }
}

