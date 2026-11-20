//! Transformadas de Fourier e filtragem básica (funcionalidade F046, RF-046).
//!
//! As funções operam sobre arrays N-D em ordem column-major descritos por `(dados, forma)`,
//! sem depender do modelo de valores do runtime. Dimensões são indexadas a partir de 0.

mod conv;
mod filter;
mod fourier;

pub use conv::{ConvShape, conv, conv2, conv2_separable, convn, deconv, filter2};
pub use filter::{FilterResult, filter, filter_along};
pub use fourier::{
    fft, fft_real, fft2, fftn, fftshift, ifft, ifft_symmetric, ifft2, ifftn, ifftshift, nextpow2,
};

use std::fmt;

/// Erros de processamento de sinais.
#[derive(Clone, Debug, PartialEq)]
pub enum SignalError {
    /// A forma não corresponde ao número de elementos.
    ShapeMismatch {
        /// Elementos esperados pela forma.
        expected: usize,
        /// Elementos recebidos.
        actual: usize,
    },
    /// Argumento inválido (descrição curta).
    InvalidArgument(&'static str),
}

impl SignalError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::ShapeMismatch { .. } => "chimalab:signal:formaIncompativel",
            Self::InvalidArgument(_) => "chimalab:signal:argumentoInvalido",
        }
    }
}

impl fmt::Display for SignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch { expected, actual } => {
                write!(
                    f,
                    "a forma indica {expected} elementos, mas foram recebidos {actual}"
                )
            }
            Self::InvalidArgument(msg) => write!(f, "argumento inválido: {msg}"),
        }
    }
}

impl std::error::Error for SignalError {}

/// Verifica `dados × forma` e devolve o número de elementos.
pub(crate) fn check_shape(len: usize, shape: &[usize]) -> Result<usize, SignalError> {
    let expected: usize = shape.iter().product();
    if expected != len {
        return Err(SignalError::ShapeMismatch {
            expected,
            actual: len,
        });
    }
    Ok(expected)
}

/// Percorre as "linhas" de um array N-D column-major ao longo de `dim`: devolve, para cada
/// linha, o índice do primeiro elemento; elementos consecutivos da linha distam `stride`.
pub(crate) fn lines(shape: &[usize], dim: usize) -> (usize, usize, Vec<usize>) {
    let stride: usize = shape[..dim].iter().product();
    let len = shape[dim];
    let outer: usize = shape[dim + 1..].iter().product();
    let mut starts = Vec::with_capacity(stride * outer);
    for o in 0..outer {
        for i in 0..stride {
            starts.push(i + o * stride * len);
        }
    }
    (stride, len, starts)
}

/// Forma estendida com dimensões singleton até incluir `dim`.
pub(crate) fn extend_shape(shape: &[usize], dim: usize) -> Vec<usize> {
    let mut s = shape.to_vec();
    while s.len() <= dim {
        s.push(1);
    }
    s
}
