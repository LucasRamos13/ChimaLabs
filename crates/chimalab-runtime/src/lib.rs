//! Modelo de valores, interpretador, workspaces e despacho (DT-32).
//!
//! Primeiro incremento: o modelo de valores com a semântica de tipos (F003), as operações
//! elemento a elemento com expansão implícita e as matriciais (F005), e a indexação na base 1
//! (F004). A sintaxe da linguagem (perguntas 27–36) não afeta estas regras.

mod error;
mod index;
mod ops;
mod value;

pub use error::RuntimeError;
pub use index::{Index, subsasgn, subsdelete, subsref};
pub use ops::{
    BinOp, CmpOp, LogicOp, Warning, colon, compare, elementwise, logical, mldivide, mtimes, neg, not, sqrt,
    transpose,
};
pub use value::{Class, IntClass, IntData, Value};
