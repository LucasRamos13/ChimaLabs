//! Funções de matriz baseadas na forma de Schur complexa: raiz quadrada e logaritmo.
//!
//! Algoritmos publicados:
//! - raiz quadrada: recorrência de Björck e Hammarling (1983) sobre a forma triangular;
//! - logaritmo: escalonamento inverso e quadratura (Kenney e Laub, 1989; Higham, 2001), com o
//!   aproximante de Padé de log(1 + x) avaliado como quadratura de Gauss-Legendre de
//!   ∫₀¹ X(I + tX)⁻¹ dt. Nós e pesos são calculados pelo método de Golub e Welsch.

use faer::linalg::triangular_solve;
use faer::{MatMut, Par};
use num_complex::Complex64;

use super::{
    ComplexMatrix, Eig, LinScalar, LinalgError, LinalgWarning, Matrix, MatrixExt, NdArray, eig, schur_complex,
};

/// Resultado que pode ser real ou complexo.
#[derive(Clone, Debug)]
pub enum RealOrComplex {
    /// Resultado real (partes imaginárias desprezíveis ou inexistentes).
    Real(Matrix),
    /// Resultado complexo.
    Complex(ComplexMatrix),
}

impl RealOrComplex {
    /// Converte para matriz complexa.
    pub fn to_complex(&self) -> ComplexMatrix {
        match self {
            Self::Real(m) => ComplexMatrix::from_real(m),
            Self::Complex(c) => c.clone(),
        }
    }
}

/// Resultado de uma função de matriz, com avisos.
#[derive(Clone, Debug)]
pub struct MatFunResult {
    /// Valor calculado.
    pub value: RealOrComplex,
    /// Avisos numéricos.
    pub warnings: Vec<LinalgWarning>,
}

/// Tolerância relativa para descartar partes imaginárias residuais de resultados reais.
const IMAG_TOL: f64 = 1e3 * f64::EPSILON;

/// Monta `U·R·Uᴴ`. Para entrada real (`allow_real`), devolve real quando as partes imaginárias
/// são desprezíveis; para entrada complexa, o resultado é sempre complexo.
fn finish(
    u: &ComplexMatrix,
    r: &ComplexMatrix,
    warnings: Vec<LinalgWarning>,
    allow_real: bool,
) -> Result<MatFunResult, LinalgError> {
    let x = u.matmul(r)?.matmul(&u.adjoint())?;
    let value = match x.to_real(IMAG_TOL).filter(|_| allow_real) {
        Some(m) => RealOrComplex::Real(m),
        None => RealOrComplex::Complex(x),
    };
    Ok(MatFunResult { value, warnings })
}

/// Raiz quadrada de matriz triangular superior complexa (ramo principal).
///
/// Coluna `j` de `R`: substituição regressiva em `(R₀₀ + r_jj·I)·x = t(0..j, j)`, orientada por
/// colunas (cada `x_i` resolvido é acumulado com a coluna `i` de `R`, contígua na memória).
fn sqrt_triangular(t: &ComplexMatrix) -> (ComplexMatrix, bool) {
    let n = t.rows();
    let zero = Complex64::new(0.0, 0.0);
    let mut r = ComplexMatrix::zeros(n, n);
    let mut singular = false;
    let ts = t.as_slice();
    let rs = r.as_mut_slice();
    let mut acc = vec![zero; n];
    for j in 0..n {
        let rjj = ts[j + j * n].sqrt();
        rs[j + j * n] = rjj;
        acc[..j].fill(zero);
        for i in (0..j).rev() {
            let den = rs[i + i * n] + rjj;
            if den.norm() == 0.0 {
                singular = true;
            }
            let x = (ts[i + j * n] - acc[i]) / den;
            rs[i + j * n] = x;
            for (a, &c) in acc[..i].iter_mut().zip(&rs[i * n..i * n + i]) {
                *a += c * x;
            }
        }
    }
    (r, singular)
}

/// Raiz quadrada principal de matriz: `X` com `X·X = A` e autovalores no semiplano direito.
///
/// Se `A` tiver autovalores reais negativos, o resultado é complexo. Matriz singular com
/// autovalor nulo repetido pode não ter raiz quadrada: emite [`LinalgWarning::Singular`].
pub fn sqrtm<T: LinScalar>(a: &NdArray<T>) -> Result<MatFunResult, LinalgError> {
    let s = schur_complex(a)?;
    let (r, singular) = sqrt_triangular(&s.t);
    let warnings = if singular {
        vec![LinalgWarning::Singular]
    } else {
        vec![]
    };
    finish(&s.u, &r, warnings, !T::IS_COMPLEX)
}

/// Nós e pesos de Gauss-Legendre em [0, 1] (Golub e Welsch: autovalores da matriz de Jacobi).
fn gauss_legendre_01(m: usize) -> Result<(Vec<f64>, Vec<f64>), LinalgError> {
    let j = Matrix::from_fn(m, m, |i, k| {
        if i.abs_diff(k) == 1 {
            let q = i.max(k) as f64;
            q / (4.0 * q * q - 1.0).sqrt()
        } else {
            0.0
        }
    });
    let Eig::Symmetric { values, vectors } = eig(&j)? else {
        unreachable!("matriz de Jacobi é simétrica")
    };
    let nodes = values.iter().map(|t| 0.5 * (t + 1.0)).collect();
    let weights = (0..m).map(|k| vectors.get(0, k).powi(2)).collect(); // 2·v₀²/2
    Ok((nodes, weights))
}

/// Resolve `M·Y = B` com `M` triangular superior complexa (substituição regressiva do faer).
fn solve_upper(m: &ComplexMatrix, b: &ComplexMatrix) -> ComplexMatrix {
    let mut y = b.clone();
    let (rows, cols) = (y.rows(), y.cols());
    let view = MatMut::from_column_major_slice_mut(y.as_mut_slice(), rows, cols);
    triangular_solve::solve_upper_triangular_in_place(m.as_faer(), view, Par::Seq);
    y
}

/// Logaritmo principal de matriz: `X` com `exp(X) = A` e autovalores com parte imaginária em
/// (−π, π].
///
/// Se `A` tiver autovalores reais negativos, o resultado é complexo. Autovalor nulo emite
/// [`LinalgWarning::Singular`] (o logaritmo não existe; o resultado contém Inf/NaN).
pub fn logm<T: LinScalar>(a: &NdArray<T>) -> Result<MatFunResult, LinalgError> {
    let s = schur_complex(a)?;
    let n = s.t.rows();
    if n == 0 {
        return Ok(MatFunResult {
            value: RealOrComplex::Real(Matrix::zeros(0, 0)),
            warnings: vec![],
        });
    }
    let mut warnings = vec![];
    if (0..n).any(|i| s.t.get(i, i).norm() == 0.0) {
        warnings.push(LinalgWarning::Singular);
    }
    let id = ComplexMatrix::identity(n);
    // Escalonamento inverso: raízes quadradas sucessivas até ‖T − I‖₁ ≤ 0,25.
    let mut t = s.t.clone();
    let mut k = 0;
    while t.sub(&id)?.norm1() > 0.25 && k < 64 {
        t = sqrt_triangular(&t).0;
        k += 1;
    }
    let x = t.sub(&id)?;
    // Padé [8/8] de log(1 + x) via Gauss-Legendre de 8 pontos (erro < 1e-19 para ‖X‖ ≤ 0,25).
    let (nodes, weights) = gauss_legendre_01(8)?;
    let mut acc = ComplexMatrix::zeros(n, n);
    for (xj, wj) in nodes.iter().zip(&weights) {
        let m = ComplexMatrix::from_fn(n, n, |i, c| id.get(i, c) + x.get(i, c) * xj);
        let y = solve_upper(&m, &x);
        acc = ComplexMatrix::from_fn(n, n, |i, c| acc.get(i, c) + y.get(i, c) * wj);
    }
    let scale = 2f64.powi(k);
    let l = ComplexMatrix::from_fn(n, n, |i, c| acc.get(i, c) * scale);
    finish(&s.u, &l, warnings, !T::IS_COMPLEX)
}
