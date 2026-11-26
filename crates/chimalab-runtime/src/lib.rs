//! Modelo de valores, interpretador, workspaces e despacho (DT-32).
//!
//! Primeiro incremento: o modelo de valores com a semântica de tipos (F003), as operações
//! elemento a elemento com expansão implícita e as matriciais (F005), e a indexação na base 1
//! (F004). A sintaxe da linguagem (perguntas 27–36) não afeta estas regras.

mod ajuda;
mod aleatorios;
mod algebra;
pub mod avaliador;
mod concat;
mod embutidas;
mod error;
mod especiais;
pub mod exibicao;
mod funcionais;
mod index;
mod numericos;
mod ops;
mod value;

pub use avaliador::{ErroExec, Interpretador, Obj, Resultado, Saida};
pub use concat::{concat, horzcat, vertcat};
pub use embutidas::todas as embutidas;
pub use error::RuntimeError;
pub use index::{Index, end_index, subsasgn, subsdelete, subsref};
pub use ops::{
    BinOp, CmpOp, LogicOp, Warning, colon, colon_elemento, colon_n, compare, elementwise, linspace, logical,
    mldivide, mpower, mrdivide, mtimes, neg, not, sqrt, transpose, verifica_tamanho,
};
pub use value::{Class, IntClass, IntData, Value};

/// Maior número de elementos de um array criado pela linguagem (2³¹; 16 GiB em `double`). Acima
/// disso, `chimalab:runtime:arrayGrandeDemais` em vez de abortar o processo por falta de memória.
pub const MAX_ELEMENTOS: usize = 1 << 31;
