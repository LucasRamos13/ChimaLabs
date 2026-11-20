//! Tipos de elemento numéricos.

use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_complex::Complex64;

/// Qualquer tipo que pode ser elemento de um [`crate::NdArray`].
pub trait Element: Clone + Debug + PartialEq + Send + Sync + 'static {}
impl<T: Clone + Debug + PartialEq + Send + Sync + 'static> Element for T {}

/// Escalar numérico de ponto flutuante de precisão dupla: real (`f64`) ou complexo (`Complex64`).
pub trait Scalar:
    Element
    + Copy
    + Default
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
    + Sum
{
    /// Verdadeiro para tipos complexos.
    const IS_COMPLEX: bool;
    /// Zero aditivo.
    fn zero() -> Self;
    /// Um multiplicativo.
    fn one() -> Self;
    /// Converte um real.
    fn from_f64(v: f64) -> Self;
    /// Parte real.
    fn re(self) -> f64;
    /// Parte imaginária (0 para reais).
    fn im(self) -> f64;
    /// Módulo.
    fn abs(self) -> f64;
    /// Conjugado (identidade para reais).
    fn conj(self) -> Self;
    /// Verdadeiro se finito (real e imaginário).
    fn is_finite(self) -> bool;
    /// Verdadeiro se NaN (em qualquer parte).
    fn is_nan(self) -> bool;
    /// Raiz quadrada principal. Para reais negativos devolve NaN.
    fn sqrt(self) -> Self;
    /// Multiplica por um real.
    fn scale(self, s: f64) -> Self;
    /// Converte para complexo.
    fn to_complex(self) -> Complex64;
    /// Converte de complexo, se representável (reais exigem parte imaginária nula).
    fn from_complex(z: Complex64) -> Option<Self>;
    /// `x/|x|` (1 para zero): "sinal" usado por estimadores de norma.
    fn signum(self) -> Self {
        let a = self.abs();
        if a == 0.0 {
            Self::one()
        } else {
            self.scale(1.0 / a)
        }
    }
}

impl Scalar for f64 {
    const IS_COMPLEX: bool = false;
    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_f64(v: f64) -> Self {
        v
    }
    fn re(self) -> f64 {
        self
    }
    fn im(self) -> f64 {
        0.0
    }
    fn abs(self) -> f64 {
        f64::abs(self)
    }
    fn conj(self) -> Self {
        self
    }
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
    fn is_nan(self) -> bool {
        f64::is_nan(self)
    }
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    fn scale(self, s: f64) -> Self {
        self * s
    }
    fn to_complex(self) -> Complex64 {
        Complex64::new(self, 0.0)
    }
    fn from_complex(z: Complex64) -> Option<Self> {
        (z.im == 0.0).then_some(z.re)
    }
}

impl Scalar for Complex64 {
    const IS_COMPLEX: bool = true;
    fn zero() -> Self {
        Complex64::new(0.0, 0.0)
    }
    fn one() -> Self {
        Complex64::new(1.0, 0.0)
    }
    fn from_f64(v: f64) -> Self {
        Complex64::new(v, 0.0)
    }
    fn re(self) -> f64 {
        self.re
    }
    fn im(self) -> f64 {
        self.im
    }
    fn abs(self) -> f64 {
        self.norm()
    }
    fn conj(self) -> Self {
        Complex64::conj(&self)
    }
    fn is_finite(self) -> bool {
        Complex64::is_finite(self)
    }
    fn is_nan(self) -> bool {
        Complex64::is_nan(self)
    }
    fn sqrt(self) -> Self {
        Complex64::sqrt(self)
    }
    fn scale(self, s: f64) -> Self {
        self * s
    }
    fn to_complex(self) -> Complex64 {
        self
    }
    fn from_complex(z: Complex64) -> Option<Self> {
        Some(z)
    }
}
