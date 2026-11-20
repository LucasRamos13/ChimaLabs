//! Álgebra linear densa (funcionalidade F040, ver docs/REQUISITOS.md RF-040).
//!
//! Implementação independente sobre o backend `faer` (MIT). Convenções do ChimaLab:
//! - matrizes reais em ordem column-major ([`Matrix`]); resultados complexos em [`ComplexMatrix`];
//! - funções que podem perder precisão devolvem **avisos** ([`LinalgWarning`]) junto do resultado,
//!   em vez de imprimir; quem chama (runtime) decide como exibi-los;
//! - erros têm identificadores estáveis ([`LinalgError::id`]).

mod balance;
mod band;
mod decomp;
mod expm;
mod funcs;
mod funm;
mod matfun;
mod matrix;
mod rcond;
mod schur;
mod solve;

pub use balance::{Balance, BalanceOptions, EigOptions, EigOrder, balance, balance_with, eig_with};
pub use band::bandwidth;
pub use decomp::{Chol, Eig, Lu, Qr, Svd, chol, eig, lu, qr, singular_values, svd};
pub use expm::expm;
pub use funcs::{NormKind, cond, norm, null_space, orth, pinv, rank};
pub use funm::{Elementary, funm, funm_elementary};
pub use matfun::{MatFunResult, RealOrComplex, logm, sqrtm};
pub use matrix::{ComplexMatrix, LinScalar, Matrix, MatrixExt, NdArray, Scalar};
pub use rcond::rcond;
pub use schur::{ComplexSchur, GenEig, Hess, Ldl, Schur, geig, hess, ldl, schur, schur_complex};
pub use solve::{Solution, SolveMethod, det, inv, solve};

use std::fmt;

/// Erros de álgebra linear.
#[derive(Clone, Debug, PartialEq)]
pub enum LinalgError {
    /// Dimensões incompatíveis para a operação.
    DimensionMismatch {
        /// Operação que falhou.
        operation: &'static str,
        /// Forma do operando esquerdo.
        left: (usize, usize),
        /// Forma do operando direito.
        right: (usize, usize),
    },
    /// A operação exige matriz quadrada.
    NotSquare {
        /// Operação que falhou.
        operation: &'static str,
        /// Forma recebida.
        shape: (usize, usize),
    },
    /// A matriz não é simétrica definida positiva.
    NotPositiveDefinite,
    /// A entrada contém NaN ou Inf e a operação não os admite.
    NonFinite {
        /// Operação que falhou.
        operation: &'static str,
    },
    /// O algoritmo iterativo não convergiu.
    NoConvergence {
        /// Operação que falhou.
        operation: &'static str,
    },
    /// Argumento fora do domínio da operação (ex.: denominador nulo).
    InvalidArgument {
        /// Operação que falhou.
        operation: &'static str,
        /// Descrição curta do problema.
        reason: &'static str,
    },
}

impl LinalgError {
    /// Identificador estável no formato `componente:mensagem`.
    pub fn id(&self) -> &'static str {
        match self {
            Self::DimensionMismatch { .. } => "chimalab:linalg:dimensoesIncompativeis",
            Self::NotSquare { .. } => "chimalab:linalg:matrizNaoQuadrada",
            Self::NotPositiveDefinite => "chimalab:linalg:naoDefinidaPositiva",
            Self::NonFinite { .. } => "chimalab:linalg:valoresNaoFinitos",
            Self::NoConvergence { .. } => "chimalab:linalg:semConvergencia",
            Self::InvalidArgument { .. } => "chimalab:linalg:argumentoInvalido",
        }
    }
}

impl fmt::Display for LinalgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionMismatch {
                operation,
                left,
                right,
            } => write!(
                f,
                "{operation}: dimensões incompatíveis ({}×{} e {}×{})",
                left.0, left.1, right.0, right.1
            ),
            Self::NotSquare { operation, shape } => {
                write!(
                    f,
                    "{operation}: a matriz precisa ser quadrada (recebida {}×{})",
                    shape.0, shape.1
                )
            }
            Self::NotPositiveDefinite => write!(f, "a matriz não é simétrica definida positiva"),
            Self::NonFinite { operation } => write!(f, "{operation}: a entrada contém NaN ou Inf"),
            Self::NoConvergence { operation } => write!(f, "{operation}: o algoritmo não convergiu"),
            Self::InvalidArgument { operation, reason } => write!(f, "{operation}: {reason}"),
        }
    }
}

impl std::error::Error for LinalgError {}

/// Avisos numéricos emitidos junto de um resultado válido.
#[derive(Clone, Debug, PartialEq)]
pub enum LinalgWarning {
    /// Matriz exatamente singular (pivô nulo): o resultado pode conter Inf/NaN.
    Singular,
    /// Matriz mal condicionada: o resultado pode ser impreciso.
    NearlySingular {
        /// Estimativa do recíproco do número de condição na norma 1.
        rcond: f64,
    },
    /// Matriz retangular ou singular com posto deficiente: foi devolvida uma solução básica.
    RankDeficient {
        /// Posto numérico detectado.
        rank: usize,
        /// Tolerância usada.
        tol: f64,
    },
}

impl LinalgWarning {
    /// Identificador estável do aviso.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Singular => "chimalab:linalg:matrizSingular",
            Self::NearlySingular { .. } => "chimalab:linalg:quaseSingular",
            Self::RankDeficient { .. } => "chimalab:linalg:postoDeficiente",
        }
    }
}

impl fmt::Display for LinalgWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Singular => write!(f, "a matriz é singular; o resultado pode conter Inf ou NaN"),
            Self::NearlySingular { rcond } => write!(
                f,
                "a matriz é mal condicionada (rcond ≈ {rcond:.3e}); o resultado pode ser impreciso"
            ),
            Self::RankDeficient { rank, tol } => {
                write!(
                    f,
                    "posto deficiente (posto = {rank}, tol = {tol:.3e}); solução básica devolvida"
                )
            }
        }
    }
}
