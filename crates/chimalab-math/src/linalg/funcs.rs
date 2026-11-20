//! Normas, condição, posto, pseudo-inversa e subespaços.

use super::decomp::{singular_values, svd};
use super::rcond::norm1;
use super::{LinScalar, LinalgError, NdArray, Scalar};

/// Tipo de norma.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NormKind {
    /// Norma 1: maior soma absoluta de coluna (soma dos módulos, para vetores).
    One,
    /// Norma 2: maior valor singular (norma euclidiana, para vetores).
    Two,
    /// Norma infinito: maior soma absoluta de linha (maior módulo, para vetores).
    Inf,
    /// Norma de Frobenius.
    Fro,
    /// Norma p de vetor (`p > 0`; `p = ∞` equivale a [`NormKind::Inf`]).
    P(f64),
}

fn is_vector<T: LinScalar>(a: &NdArray<T>) -> bool {
    a.rows() == 1 || a.cols() == 1
}

/// Norma de matriz ou de vetor (vetor = matriz com uma linha ou uma coluna).
pub fn norm<T: LinScalar>(a: &NdArray<T>, kind: NormKind) -> Result<f64, LinalgError> {
    if a.is_empty() {
        return Ok(0.0);
    }
    let v: Vec<f64> = a.as_slice().iter().map(|x| x.abs()).collect();
    if a.as_slice().iter().any(|x| Scalar::is_nan(*x)) {
        return Ok(f64::NAN);
    }
    let fro = || v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if is_vector(a) {
        return Ok(match kind {
            NormKind::One => v.iter().map(|x| x.abs()).sum(),
            NormKind::Two | NormKind::Fro => hypot_all(&v),
            NormKind::Inf => a.max_abs(),
            NormKind::P(p) if p == f64::INFINITY => a.max_abs(),
            NormKind::P(p) if p == f64::NEG_INFINITY => {
                v.iter().map(|x| x.abs()).fold(f64::INFINITY, f64::min)
            }
            NormKind::P(p) => v.iter().map(|x| x.abs().powf(p)).sum::<f64>().powf(1.0 / p),
        });
    }
    match kind {
        NormKind::One => Ok(norm1(a)),
        NormKind::Inf => Ok((0..a.rows())
            .map(|i| (0..a.cols()).map(|j| Scalar::abs(a.get(i, j))).sum::<f64>())
            .fold(0.0, f64::max)),
        NormKind::Fro => Ok(fro()),
        NormKind::Two => {
            if !a.all_finite() {
                return Ok(f64::INFINITY);
            }
            Ok(singular_values(a)?.first().copied().unwrap_or(0.0))
        }
        NormKind::P(_) => Err(LinalgError::DimensionMismatch {
            operation: "norm (p só é definida para vetores)",
            left: a.shape(),
            right: (1, 1),
        }),
    }
}

/// Norma euclidiana com escalonamento para evitar overflow/underflow.
fn hypot_all(v: &[f64]) -> f64 {
    let scale = v.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
    if scale == 0.0 || !scale.is_finite() {
        return scale;
    }
    scale * v.iter().map(|x| (x / scale).powi(2)).sum::<f64>().sqrt()
}

/// Número de condição na norma 2 (`σ_max / σ_min`). Matriz singular devolve `Inf`; vazia, 0.
pub fn cond<T: LinScalar>(a: &NdArray<T>) -> Result<f64, LinalgError> {
    if a.is_empty() {
        return Ok(0.0);
    }
    let s = singular_values(a)?;
    let (smax, smin) = (s[0], *s.last().unwrap_or(&0.0));
    Ok(if smin == 0.0 { f64::INFINITY } else { smax / smin })
}

fn default_tol<T: LinScalar>(a: &NdArray<T>, smax: f64) -> f64 {
    (a.rows().max(a.cols()) as f64) * smax * f64::EPSILON
}

/// Posto numérico: quantidade de valores singulares maiores que `tol`
/// (padrão: `max(m, n) · σ_max · ε`).
pub fn rank<T: LinScalar>(a: &NdArray<T>, tol: Option<f64>) -> Result<usize, LinalgError> {
    if a.is_empty() {
        return Ok(0);
    }
    let s = singular_values(a)?;
    let tol = tol.unwrap_or_else(|| default_tol(a, s[0]));
    Ok(s.iter().filter(|&&x| x > tol).count())
}

/// Pseudo-inversa de Moore-Penrose (via SVD), com a mesma tolerância de [`rank`].
pub fn pinv<T: LinScalar>(a: &NdArray<T>, tol: Option<f64>) -> Result<NdArray<T>, LinalgError> {
    let (m, n) = a.shape();
    if a.is_empty() {
        return Ok(NdArray::zeros(n, m));
    }
    let f = svd(a, true)?;
    let tol = tol.unwrap_or_else(|| default_tol(a, f.s[0]));
    let k = f.s.len();
    Ok(NdArray::from_fn(n, m, |i, j| {
        (0..k)
            .filter(|&p| f.s[p] > tol)
            .map(|p| (f.v.get(i, p) * Scalar::conj(f.u.get(j, p))).scale(1.0 / f.s[p]))
            .sum()
    }))
}

/// Base ortonormal do espaço nulo de `a` (colunas).
pub fn null_space<T: LinScalar>(a: &NdArray<T>) -> Result<NdArray<T>, LinalgError> {
    let n = a.cols();
    if a.is_empty() {
        return Ok(NdArray::identity(n));
    }
    let f = svd(a, false)?;
    let r = f.s.iter().filter(|&&x| x > default_tol(a, f.s[0])).count();
    Ok(f.v.submatrix(0, n, r, n))
}

/// Base ortonormal da imagem (espaço coluna) de `a`.
pub fn orth<T: LinScalar>(a: &NdArray<T>) -> Result<NdArray<T>, LinalgError> {
    let m = a.rows();
    if a.is_empty() {
        return Ok(NdArray::zeros(m, 0));
    }
    let f = svd(a, true)?;
    let r = f.s.iter().filter(|&&x| x > default_tol(a, f.s[0])).count();
    Ok(f.u.submatrix(0, m, 0, r))
}
