//! Tipos de matriz da álgebra linear: apelidos do array N-D genérico (DT-18).

use chimalab_array::ArrayError;
use faer::{Mat, MatRef};
use num_complex::Complex64;

pub use chimalab_array::{NdArray, Scalar};

use super::LinalgError;

/// Matriz real (array 2-D de `f64`).
pub type Matrix = NdArray<f64>;
/// Matriz complexa (array 2-D de `Complex64`).
pub type ComplexMatrix = NdArray<Complex64>;

/// Escalar aceito pelas rotinas de álgebra linear: `f64` ou `Complex64`.
pub trait LinScalar: Scalar + faer::traits::ComplexField<Real = f64> {}
impl LinScalar for f64 {}
impl LinScalar for Complex64 {}

/// Operações matriciais que dependem do backend numérico.
pub trait MatrixExt<T: LinScalar>: Sized {
    /// Produto matricial `self · rhs` (visões 2-D).
    fn matmul(&self, rhs: &Self) -> Result<Self, LinalgError>;
    /// Visão do backend (exige array 2-D).
    fn as_faer(&self) -> MatRef<'_, T>;
    /// Cópia a partir de uma visão do backend.
    fn from_faer(m: MatRef<'_, T>) -> Self;
    /// Cópia para o tipo do backend.
    fn to_faer(&self) -> Mat<T>;
}

impl<T: LinScalar> MatrixExt<T> for NdArray<T> {
    fn matmul(&self, rhs: &Self) -> Result<Self, LinalgError> {
        if self.cols() != rhs.rows() {
            return Err(LinalgError::DimensionMismatch {
                operation: "matmul",
                left: self.shape(),
                right: rhs.shape(),
            });
        }
        if self.is_empty() || rhs.is_empty() {
            return Ok(NdArray::zeros(self.rows(), rhs.cols()));
        }
        Ok(Self::from_faer((self.as_faer() * rhs.as_faer()).as_ref()))
    }

    fn as_faer(&self) -> MatRef<'_, T> {
        MatRef::from_column_major_slice(self.as_slice(), self.rows(), self.cols())
    }

    fn from_faer(m: MatRef<'_, T>) -> Self {
        NdArray::from_fn(m.nrows(), m.ncols(), |i, j| m[(i, j)])
    }

    fn to_faer(&self) -> Mat<T> {
        Mat::from_fn(self.rows(), self.cols(), |i, j| self.get(i, j))
    }
}

impl From<ArrayError> for LinalgError {
    fn from(e: ArrayError) -> Self {
        let two = |d: &[usize]| {
            (
                d.first().copied().unwrap_or(1),
                d[1.min(d.len())..].iter().product(),
            )
        };
        match e {
            ArrayError::IncompatibleShapes {
                operation,
                left,
                right,
            } => LinalgError::DimensionMismatch {
                operation,
                left: two(&left),
                right: two(&right),
            },
            ArrayError::ShapeMismatch { expected, actual } => LinalgError::DimensionMismatch {
                operation: "forma",
                left: (expected, 1),
                right: (actual, 1),
            },
            ArrayError::InvalidPermutation(_) | ArrayError::IndexOutOfBounds { .. } => {
                LinalgError::DimensionMismatch {
                    operation: "índice",
                    left: (0, 0),
                    right: (0, 0),
                }
            }
        }
    }
}
