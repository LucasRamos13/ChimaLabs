//! Erros do runtime.

use std::fmt;

/// Erros com identificador estável.
#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeError {
    /// Combinação de classes não permitida (ex.: int8 com int16, inteiro com complexo).
    IncompatibleClasses {
        /// Operação.
        operation: &'static str,
        /// Classe à esquerda.
        left: String,
        /// Classe à direita.
        right: String,
    },
    /// Dimensões incompatíveis.
    IncompatibleShapes {
        /// Operação.
        operation: &'static str,
        /// Forma à esquerda.
        left: Vec<usize>,
        /// Forma à direita.
        right: Vec<usize>,
    },
    /// Índice fora dos limites na leitura.
    IndexOutOfBounds {
        /// Índice pedido (base 1).
        index: usize,
        /// Limite.
        bound: usize,
    },
    /// Índice inválido (zero, não inteiro, lógico com tamanho errado, etc.).
    InvalidIndex(String),
    /// Conversão impossível (ex.: NaN para lógico, complexo para inteiro).
    InvalidConversion(String),
    /// Erro numérico vindo da biblioteca matemática.
    Math(String),
}

impl RuntimeError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::IncompatibleClasses { .. } => "chimalab:runtime:classesIncompativeis",
            Self::IncompatibleShapes { .. } => "chimalab:runtime:dimensoesIncompativeis",
            Self::IndexOutOfBounds { .. } => "chimalab:runtime:indiceForaDosLimites",
            Self::InvalidIndex(_) => "chimalab:runtime:indiceInvalido",
            Self::InvalidConversion(_) => "chimalab:runtime:conversaoInvalida",
            Self::Math(_) => "chimalab:runtime:erroNumerico",
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleClasses {
                operation,
                left,
                right,
            } => {
                write!(f, "{operation}: classes incompatíveis {left} e {right}")
            }
            Self::IncompatibleShapes {
                operation,
                left,
                right,
            } => {
                write!(f, "{operation}: dimensões incompatíveis {left:?} e {right:?}")
            }
            Self::IndexOutOfBounds { index, bound } => {
                write!(f, "índice {index} fora dos limites (máximo {bound})")
            }
            Self::InvalidIndex(m) => write!(f, "índice inválido: {m}"),
            Self::InvalidConversion(m) => write!(f, "conversão inválida: {m}"),
            Self::Math(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for RuntimeError {}
