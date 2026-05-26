//! Register-based intermediate representation for the Vela runtime.
//!
//! See `docs/ARCHITECTURE.md` for the rationale and how this fits into
//! the VM/JIT pipeline. This crate owns the data structures only; the
//! compiler that targets this IR lives elsewhere, and so does the VM
//! that runs it.

use std::ops::Range;

pub type Reg = u16;
pub type ConstIdx = u32;
pub type FunctionId = u32;
pub type Offset = i32;
pub type UpvalIdx = u16;
pub type CtorIdx = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Const {
    Int(i64),
    UInt(u64),
    BigInt(String),
    Float(f64),
    Decimal(String),
    Str(String),
    Bool(bool),
    Sym(String),
    Unit,
    FieldName(String),
    FieldNames(Vec<String>),
    CtorName(String),
    GlobalName(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    LoadConst {
        dst: Reg,
        k: ConstIdx,
    },
    Move {
        dst: Reg,
        src: Reg,
    },

    Add {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Sub {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Mul {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Div {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Mod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Pow {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Concat {
        dst: Reg,
        a: Reg,
        b: Reg,
    },

    Eq {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Ne {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Lt {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Le {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Gt {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Ge {
        dst: Reg,
        a: Reg,
        b: Reg,
    },

    Neg {
        dst: Reg,
        a: Reg,
    },
    Not {
        dst: Reg,
        a: Reg,
    },

    Jump {
        offset: Offset,
    },
    JumpIfFalse {
        cond: Reg,
        offset: Offset,
    },

    Call {
        dst: Reg,
        callee: Reg,
        base: Reg,
        nargs: u16,
    },
    Return {
        src: Reg,
    },

    MkClosure {
        dst: Reg,
        function: FunctionId,
        n_upvals: u16,
    },
    GetUpval {
        dst: Reg,
        idx: UpvalIdx,
    },
    SetUpval {
        idx: UpvalIdx,
        src: Reg,
    },
    CloseUpvals {
        from: Reg,
    },

    GetGlobal {
        dst: Reg,
        name: ConstIdx,
    },
    SetGlobal {
        name: ConstIdx,
        src: Reg,
    },

    MkTuple {
        dst: Reg,
        base: Reg,
        n: u16,
    },
    MkSeries {
        dst: Reg,
        base: Reg,
        n: u16,
    },
    MkRecord {
        dst: Reg,
        base: Reg,
        n: u16,
        names: ConstIdx,
    },
    MkCons {
        dst: Reg,
        ctor: CtorIdx,
        base: Reg,
        n: u16,
    },

    GetField {
        dst: Reg,
        obj: Reg,
        name: ConstIdx,
    },
    SetField {
        obj: Reg,
        name: ConstIdx,
        src: Reg,
    },
    GetIndex {
        dst: Reg,
        seq: Reg,
        idx: Reg,
    },

    QuestionUnwrap {
        dst: Reg,
        src: Reg,
    },

    IsCons {
        dst: Reg,
        scrut: Reg,
        name: ConstIdx,
    },
    ConsArg {
        dst: Reg,
        src: Reg,
        idx: u16,
    },
    Panic {
        msg: ConstIdx,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpvalDesc {
    pub from_parent_local: bool,
    pub index: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: u16,
    pub n_regs: u16,
    pub n_upvals: u16,
    pub upvals: Vec<UpvalDesc>,
    pub consts: Vec<Const>,
    pub code: Vec<Op>,
    pub source_path: String,
    pub source_spans: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Module {
    pub functions: Vec<Function>,
    pub entry: Option<FunctionId>,
}

impl Module {
    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[id as usize]
    }
}

impl Function {
    pub fn instruction_count(&self) -> usize {
        self.code.len()
    }
}
