//! Integração numérica e quadratura (funcionalidade F048, RF-048).
//!
//! - [`integral`]: quadratura adaptativa global de Gauss–Kronrod 7–15 com tolerâncias absoluta e
//!   relativa, limites infinitos, singularidades integráveis nos extremos e pontos de quebra;
//!   integrandos reais ou complexos.
//! - [`integral2`] e [`integral3`]: integrais iteradas em regiões com limites internos variáveis.
//! - [`trapz`] e [`cumtrapz`]: regra trapezoidal sobre dados amostrados ao longo de qualquer
//!   dimensão de um array N-D column-major `(dados, forma)`.
//!
//! Fontes: A. S. Kronrod, *Nodes and Weights of Quadrature Formulas* (1965), para a construção da
//! regra 7–15 (nós e pesos derivados em `ferramentas/oraculos/f048_gauss_kronrod.py`); L. F.
//! Shampine, J. Comput. Appl. Math. 211 (2008)
//! 131–140, para as mudanças de variável nos extremos e em intervalos infinitos.

mod adaptive;
mod gk;
mod multi;
mod trapz;

pub use adaptive::{IntegralOptions, IntegralResult, IntegralWarning, integral};
pub use multi::{integral2, integral2_tiled, integral3};
pub use trapz::{Spacing, cumtrapz, trapz};

use std::fmt;

/// Erros de integração.
#[derive(Clone, Debug, PartialEq)]
pub enum IntegrateError {
    /// Limite NaN.
    InvalidLimits(&'static str),
    /// Tolerância negativa, NaN ou infinita.
    InvalidTolerance(&'static str),
    /// A forma não corresponde ao número de elementos, ou o vetor de abscissas não tem o
    /// comprimento da dimensão integrada.
    ShapeMismatch {
        /// Elementos esperados.
        expected: usize,
        /// Elementos recebidos.
        actual: usize,
    },
}

impl IntegrateError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::InvalidLimits(_) => "chimalab:integral:limitesInvalidos",
            Self::InvalidTolerance(_) => "chimalab:integral:toleranciaInvalida",
            Self::ShapeMismatch { .. } => "chimalab:integral:formaIncompativel",
        }
    }
}

impl fmt::Display for IntegrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimits(m) => write!(f, "limites de integração inválidos: {m}"),
            Self::InvalidTolerance(m) => write!(f, "tolerância inválida: {m}"),
            Self::ShapeMismatch { expected, actual } => {
                write!(
                    f,
                    "eram esperados {expected} elementos, mas foram recebidos {actual}"
                )
            }
        }
    }
}

impl std::error::Error for IntegrateError {}
