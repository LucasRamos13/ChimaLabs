//! Array N-D genérico do ChimaLab (DT-18).
//!
//! - Ordem **column-major**: o primeiro índice varia mais rápido.
//! - Forma com **pelo menos 2 dimensões**; singletons finais além da segunda são removidos
//!   (`[3]` vira `[3, 1]`, `[2, 3, 1, 1]` vira `[2, 3]`).
//! - Dados compartilhados em `Arc` com **cópia sob demanda**: clonar é O(1); a primeira
//!   escrita em uma cópia compartilhada duplica os dados.
//! - Índices a partir de 0 (a tradução de índices da linguagem fica no runtime).

mod array;
mod error;
mod scalar;

pub use array::{NdArray, broadcast_shape, normalize_shape};
pub use error::ArrayError;
pub use num_complex::Complex64;
pub use scalar::{Element, Scalar};
