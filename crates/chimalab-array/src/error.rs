//! Erros de arrays.

use std::fmt;

/// Erros de construção e manipulação de arrays.
#[derive(Clone, Debug, PartialEq)]
pub enum ArrayError {
    /// A forma indica um número de elementos diferente do recebido.
    ShapeMismatch {
        /// Elementos indicados pela forma.
        expected: usize,
        /// Elementos recebidos.
        actual: usize,
    },
    /// Formas incompatíveis para a operação (ex.: expansão implícita).
    IncompatibleShapes {
        /// Operação.
        operation: &'static str,
        /// Forma do primeiro operando.
        left: Vec<usize>,
        /// Forma do segundo operando.
        right: Vec<usize>,
    },
    /// Permutação de dimensões inválida.
    InvalidPermutation(Vec<usize>),
    /// Índice fora dos limites.
    IndexOutOfBounds {
        /// Índice pedido.
        index: Vec<usize>,
        /// Forma do array.
        shape: Vec<usize>,
    },
}

impl ArrayError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::ShapeMismatch { .. } => "chimalab:array:formaIncompativel",
            Self::IncompatibleShapes { .. } => "chimalab:array:dimensoesIncompativeis",
            Self::InvalidPermutation(_) => "chimalab:array:permutacaoInvalida",
            Self::IndexOutOfBounds { .. } => "chimalab:array:indiceForaDosLimites",
        }
    }
}

impl fmt::Display for ArrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch { expected, actual } => {
                write!(
                    f,
                    "a forma indica {expected} elementos, mas foram recebidos {actual}"
                )
            }
            Self::IncompatibleShapes {
                operation,
                left,
                right,
            } => {
                write!(f, "{operation}: formas incompatíveis {left:?} e {right:?}")
            }
            Self::InvalidPermutation(p) => write!(f, "permutação de dimensões inválida: {p:?}"),
            Self::IndexOutOfBounds { index, shape } => {
                write!(f, "índice {index:?} fora dos limites da forma {shape:?}")
            }
        }
    }
}

impl std::error::Error for ArrayError {}
