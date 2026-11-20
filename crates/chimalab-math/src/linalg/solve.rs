//! Solução de sistemas lineares com escolha automática de método, inversa e determinante.

use faer::linalg::solvers::{PartialPivLu, Solve};
use faer::{MatMut, Side};

use super::band::{BandChol, BandLu, bandwidth};
use super::rcond::{norm1, norm1_estimate};
use super::{LinScalar, LinalgError, LinalgWarning, MatrixExt, NdArray, Scalar};

/// Método escolhido por [`solve`] a partir da estrutura da matriz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolveMethod {
    /// Matriz de coeficientes sem elementos.
    Empty,
    /// Substituição regressiva (triangular superior).
    UpperTriangular,
    /// Substituição progressiva (triangular inferior).
    LowerTriangular,
    /// Fatoração de Cholesky (simétrica definida positiva).
    Cholesky,
    /// Fatoração LU com pivotamento parcial.
    Lu,
    /// Cholesky em armazenamento de banda (hermitiana definida positiva com banda estreita).
    BandCholesky,
    /// LU de banda com pivotamento parcial (banda estreita).
    BandLu,
    /// Mínimos quadrados por QR com pivotamento de colunas (sistemas retangulares).
    LeastSquaresQr,
}

/// Resultado de [`solve`].
#[derive(Clone, Debug)]
pub struct Solution<T: LinScalar> {
    /// Solução `x` de `A x = B`.
    pub x: NdArray<T>,
    /// Método usado.
    pub method: SolveMethod,
    /// Avisos numéricos.
    pub warnings: Vec<LinalgWarning>,
}

/// Limite de `rcond` abaixo do qual o sistema é considerado mal condicionado.
const RCOND_LIMIT: f64 = f64::EPSILON;

/// Ordem mínima para usar os solvers de banda: abaixo dela a fatoração densa é igualmente rápida.
pub(crate) const BAND_MIN_N: usize = 32;

/// A banda é "estreita" (vale a pena o armazenamento compacto) quando `kl + ku + 1 ≤ n/4`.
pub(crate) fn is_narrow_band(n: usize, kl: usize, ku: usize) -> bool {
    n >= BAND_MIN_N && 4 * (kl + ku + 1) <= n
}

/// Resolve `A x = B` (divisão à esquerda).
///
/// Aceita matrizes reais ou complexas.
///
/// - `A` quadrada: escolhe triangular, Cholesky (se hermitiana com diagonal real positiva e a
///   fatoração tiver sucesso) ou LU com pivotamento parcial. Se a banda for estreita
///   (`n ≥ 32` e `kl + ku + 1 ≤ n/4`), Cholesky e LU usam armazenamento de banda
///   ([`SolveMethod::BandCholesky`], [`SolveMethod::BandLu`]); emite [`LinalgWarning::Singular`] ou
///   [`LinalgWarning::NearlySingular`] conforme a estimativa de condição.
/// - `A` retangular: solução de mínimos quadrados por QR com pivotamento de colunas; com posto
///   deficiente, devolve a solução básica (com no máximo `posto` componentes não nulas) e emite
///   [`LinalgWarning::RankDeficient`].
pub fn solve<T: LinScalar>(a: &NdArray<T>, b: &NdArray<T>) -> Result<Solution<T>, LinalgError> {
    if a.rows() != b.rows() {
        return Err(LinalgError::DimensionMismatch {
            operation: "solve",
            left: a.shape(),
            right: b.shape(),
        });
    }
    if a.is_empty() {
        return Ok(Solution {
            x: NdArray::zeros(a.cols(), b.cols()),
            method: SolveMethod::Empty,
            warnings: vec![],
        });
    }
    if !a.all_finite() || !b.all_finite() {
        return Err(LinalgError::NonFinite { operation: "solve" });
    }
    if !a.is_square() {
        return least_squares(a, b);
    }
    if a.is_upper_triangular() {
        return Ok(triangular(a, b, true));
    }
    if a.is_lower_triangular() {
        return Ok(triangular(a, b, false));
    }
    let n = a.rows();
    let (kl, ku) = bandwidth(a);
    let band = is_narrow_band(n, kl, ku);
    if a.is_hermitian() && (0..n).all(|i| a.get(i, i).re() > 0.0) {
        let sol = if band {
            band_cholesky_solve(a, b, ku)
        } else {
            cholesky_solve(a, b)
        };
        if let Some(sol) = sol {
            return Ok(sol);
        }
    }
    if band && let Some(f) = BandLu::factor(a, kl, ku) {
        let sa = |v: &mut [T]| f.solve_in_place(v);
        let sat = |v: &mut [T]| f.solve_adjoint_in_place(v);
        let x = solve_columns(b, &sa);
        let rc = reciprocal(norm1(a), norm1_estimate(n, &sa, &sat));
        return Ok(Solution {
            x,
            method: SolveMethod::BandLu,
            warnings: conditioning_warnings(rc),
        });
    }
    // Banda larga, ou pivô nulo na LU de banda (singular): LU densa, que diagnostica o caso.
    let f = lu_factor(a);
    let mut x = b.to_faer();
    f.lu.solve_in_place(&mut x);
    Ok(Solution {
        x: NdArray::from_faer(x.as_ref()),
        method: SolveMethod::Lu,
        warnings: f.warnings(),
    })
}

/// Fatoração LU com diagnóstico de singularidade e estimativa de condição.
pub(crate) struct LuFactor<T: LinScalar> {
    pub(crate) lu: PartialPivLu<T>,
    pub(crate) singular: bool,
    pub(crate) rcond: f64,
}

impl<T: LinScalar> LuFactor<T> {
    fn warnings(&self) -> Vec<LinalgWarning> {
        if self.singular {
            vec![LinalgWarning::Singular]
        } else if self.rcond < RCOND_LIMIT {
            vec![LinalgWarning::NearlySingular { rcond: self.rcond }]
        } else {
            vec![]
        }
    }
}

pub(crate) fn lu_factor<T: LinScalar>(a: &NdArray<T>) -> LuFactor<T> {
    let n = a.rows();
    let lu = a.as_faer().partial_piv_lu();
    let u = lu.U();
    let singular = (0..n).any(|i| u[(i, i)] == T::zero());
    let rcond = if singular {
        0.0
    } else {
        let solve_a = |v: &mut [T]| lu.solve_in_place(MatMut::from_column_major_slice_mut(v, n, 1));
        let solve_at = |v: &mut [T]| lu.solve_adjoint_in_place(MatMut::from_column_major_slice_mut(v, n, 1));
        reciprocal(norm1(a), norm1_estimate(n, &solve_a, &solve_at))
    };
    LuFactor { lu, singular, rcond }
}

fn reciprocal(anorm: f64, ainv_norm: f64) -> f64 {
    if anorm == 0.0 || !ainv_norm.is_finite() || ainv_norm == 0.0 {
        0.0
    } else {
        (1.0 / (anorm * ainv_norm)).min(1.0)
    }
}

/// Resolve sistema triangular no lugar (coluna `x` de comprimento n). Com `adjoint`, resolve
/// com `Tᴴ`.
pub(crate) fn tri_solve_in_place<T: LinScalar>(t: &NdArray<T>, x: &mut [T], upper: bool, adjoint: bool) {
    let n = t.rows();
    // Com transposição, um sistema triangular superior vira inferior e vice-versa.
    let backward = upper != adjoint;
    let at = |i: usize, j: usize| if adjoint { t.get(j, i).conj() } else { t.get(i, j) };
    if backward {
        for i in (0..n).rev() {
            let s = x[i] - (i + 1..n).map(|j| at(i, j) * x[j]).sum::<T>();
            x[i] = s / at(i, i);
        }
    } else {
        for i in 0..n {
            let s = x[i] - (0..i).map(|j| at(i, j) * x[j]).sum::<T>();
            x[i] = s / at(i, i);
        }
    }
}

fn triangular<T: LinScalar>(a: &NdArray<T>, b: &NdArray<T>, upper: bool) -> Solution<T> {
    let n = a.rows();
    let mut x = b.clone();
    let mut col = vec![T::zero(); n];
    for k in 0..b.cols() {
        for (i, c) in col.iter_mut().enumerate() {
            *c = x.get(i, k);
        }
        tri_solve_in_place(a, &mut col, upper, false);
        for (i, c) in col.iter().enumerate() {
            x.set(i, k, *c);
        }
    }
    let singular = (0..n).any(|i| a.get(i, i) == T::zero());
    let warnings = if singular {
        vec![LinalgWarning::Singular]
    } else {
        let sa = |v: &mut [T]| tri_solve_in_place(a, v, upper, false);
        let sat = |v: &mut [T]| tri_solve_in_place(a, v, upper, true);
        let rc = reciprocal(norm1(a), norm1_estimate(n, &sa, &sat));
        if rc < RCOND_LIMIT {
            vec![LinalgWarning::NearlySingular { rcond: rc }]
        } else {
            vec![]
        }
    };
    let method = if upper {
        SolveMethod::UpperTriangular
    } else {
        SolveMethod::LowerTriangular
    };
    Solution { x, method, warnings }
}

/// Aplica `solve_col` a cada coluna de `b`.
fn solve_columns<T: LinScalar>(b: &NdArray<T>, solve_col: &dyn Fn(&mut [T])) -> NdArray<T> {
    let n = b.rows();
    let mut x = b.clone();
    for c in x.as_mut_slice().chunks_mut(n.max(1)) {
        solve_col(c);
    }
    x
}

fn conditioning_warnings(rc: f64) -> Vec<LinalgWarning> {
    if rc < RCOND_LIMIT {
        vec![LinalgWarning::NearlySingular { rcond: rc }]
    } else {
        vec![]
    }
}

fn band_cholesky_solve<T: LinScalar>(a: &NdArray<T>, b: &NdArray<T>, kd: usize) -> Option<Solution<T>> {
    let f = BandChol::factor(a, kd)?;
    let sa = |v: &mut [T]| f.solve_in_place(v);
    let x = solve_columns(b, &sa);
    // A é hermitiana: Aᴴ = A.
    let rc = reciprocal(norm1(a), norm1_estimate(a.rows(), &sa, &sa));
    Some(Solution {
        x,
        method: SolveMethod::BandCholesky,
        warnings: conditioning_warnings(rc),
    })
}

fn cholesky_solve<T: LinScalar>(a: &NdArray<T>, b: &NdArray<T>) -> Option<Solution<T>> {
    let n = a.rows();
    let llt = a.as_faer().llt(Side::Lower).ok()?;
    let mut x = b.to_faer();
    llt.solve_in_place(&mut x);
    let sa = |v: &mut [T]| llt.solve_in_place(MatMut::from_column_major_slice_mut(v, n, 1));
    // A é hermitiana: Aᴴ = A.
    let rc = reciprocal(norm1(a), norm1_estimate(n, &sa, &sa));
    let warnings = if rc < RCOND_LIMIT {
        vec![LinalgWarning::NearlySingular { rcond: rc }]
    } else {
        vec![]
    };
    Some(Solution {
        x: NdArray::from_faer(x.as_ref()),
        method: SolveMethod::Cholesky,
        warnings,
    })
}

fn least_squares<T: LinScalar>(a: &NdArray<T>, b: &NdArray<T>) -> Result<Solution<T>, LinalgError> {
    let (m, n) = a.shape();
    let qr = a.as_faer().col_piv_qr();
    let k = m.min(n);
    let r = qr.thin_R();
    let q = qr.compute_thin_Q();
    let perm = qr.P();
    let fwd = perm.arrays().0;
    let r00 = Scalar::abs(r[(0, 0)]);
    let tol = (m.max(n) as f64) * f64::EPSILON * r00;
    let rank = (0..k).take_while(|&i| Scalar::abs(r[(i, i)]) > tol).count();
    // y = Qᴴ b (k × nrhs)
    let y = q.adjoint() * b.as_faer();
    let mut x = NdArray::zeros(n, b.cols());
    let rr = NdArray::from_fn(rank, rank, |i, j| r[(i, j)]);
    for c in 0..b.cols() {
        let mut z: Vec<T> = (0..rank).map(|i| y[(i, c)]).collect();
        tri_solve_in_place(&rr, &mut z, true, false);
        for (j, zj) in z.into_iter().enumerate() {
            x.set(fwd[j], c, zj);
        }
    }
    let warnings = if rank < k {
        vec![LinalgWarning::RankDeficient { rank, tol }]
    } else {
        vec![]
    };
    Ok(Solution {
        x,
        method: SolveMethod::LeastSquaresQr,
        warnings,
    })
}

/// Inversa de uma matriz quadrada, com os mesmos avisos de [`solve`].
///
/// Para resolver sistemas prefira [`solve`], que é mais preciso e mais rápido.
pub fn inv<T: LinScalar>(a: &NdArray<T>) -> Result<(NdArray<T>, Vec<LinalgWarning>), LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "inv",
            shape: a.shape(),
        });
    }
    let s = solve(a, &NdArray::identity(a.rows()))?;
    Ok((s.x, s.warnings))
}

/// Determinante de uma matriz quadrada (via LU). O determinante da matriz vazia é 1.
pub fn det<T: LinScalar>(a: &NdArray<T>) -> Result<T, LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "det",
            shape: a.shape(),
        });
    }
    if a.is_empty() {
        return Ok(T::one());
    }
    if !a.all_finite() {
        return Err(LinalgError::NonFinite { operation: "det" });
    }
    let n = a.rows();
    let lu = a.as_faer().partial_piv_lu();
    let u = lu.U();
    let l = lu.L();
    let mut d = (0..n).fold(T::one(), |acc, i| acc * u[(i, i)] * l[(i, i)]);
    if permutation_is_odd(lu.P().arrays().0) {
        d = -d;
    }
    Ok(d)
}

/// Paridade de uma permutação pela contagem de ciclos.
pub(crate) fn permutation_is_odd(p: &[usize]) -> bool {
    let mut seen = vec![false; p.len()];
    let mut transpositions = 0;
    for start in 0..p.len() {
        if seen[start] {
            continue;
        }
        let mut len = 0;
        let mut i = start;
        while !seen[i] {
            seen[i] = true;
            i = p[i];
            len += 1;
        }
        transpositions += len - 1;
    }
    transpositions % 2 == 1
}
