//! Decomposições matriciais: LU, QR, Cholesky, SVD e autovalores (reais ou complexas).

use faer::Side;
use num_complex::Complex64;

use super::{ComplexMatrix, LinScalar, LinalgError, MatrixExt, NdArray, Scalar};

/// Decomposição `P·A = L·U` com pivotamento parcial de linhas.
#[derive(Clone, Debug)]
pub struct Lu<T: LinScalar> {
    /// Fator triangular inferior com diagonal unitária (m × k, k = min(m, n)).
    pub l: NdArray<T>,
    /// Fator triangular superior (k × n).
    pub u: NdArray<T>,
    /// Permutação de linhas: a linha `i` de `P·A` é a linha `perm[i]` de `A`.
    pub perm: Vec<usize>,
}

impl<T: LinScalar> Lu<T> {
    /// Matriz de permutação `P` explícita.
    pub fn p_matrix(&self) -> NdArray<T> {
        let n = self.perm.len();
        NdArray::from_fn(n, n, |i, j| if self.perm[i] == j { T::one() } else { T::zero() })
    }
}

/// Decomposição `A = Q·R` completa.
#[derive(Clone, Debug)]
pub struct Qr<T: LinScalar> {
    /// Fator unitário (m × m; ortogonal no caso real).
    pub q: NdArray<T>,
    /// Fator triangular superior (m × n).
    pub r: NdArray<T>,
}

/// Decomposição de Cholesky `A = Rᴴ·R` (`Rᵀ·R` no caso real).
#[derive(Clone, Debug)]
pub struct Chol<T: LinScalar> {
    /// Fator triangular superior.
    pub r: NdArray<T>,
}

/// Decomposição em valores singulares `A = U·diag(S)·Vᴴ`.
#[derive(Clone, Debug)]
pub struct Svd<T: LinScalar> {
    /// Vetores singulares à esquerda (m × m, ou m × k na versão econômica).
    pub u: NdArray<T>,
    /// Valores singulares em ordem não crescente (k = min(m, n)).
    pub s: Vec<f64>,
    /// Vetores singulares à direita (n × n, ou n × k na versão econômica).
    pub v: NdArray<T>,
}

/// Decomposição em autovalores `A·V = V·diag(λ)`.
#[derive(Clone, Debug)]
pub enum Eig<T: LinScalar> {
    /// Matriz hermitiana (simétrica, no caso real): autovalores reais em ordem não decrescente e
    /// autovetores ortonormais.
    Symmetric {
        /// Autovalores.
        values: Vec<f64>,
        /// Autovetores (colunas).
        vectors: NdArray<T>,
    },
    /// Caso geral: autovalores e autovetores complexos.
    General {
        /// Autovalores.
        values: Vec<Complex64>,
        /// Autovetores (colunas, norma 2 unitária).
        vectors: ComplexMatrix,
    },
}

fn require_finite<T: LinScalar>(a: &NdArray<T>, operation: &'static str) -> Result<(), LinalgError> {
    if a.all_finite() {
        Ok(())
    } else {
        Err(LinalgError::NonFinite { operation })
    }
}

/// Fatoração LU com pivotamento parcial (aceita matrizes retangulares).
pub fn lu<T: LinScalar>(a: &NdArray<T>) -> Result<Lu<T>, LinalgError> {
    require_finite(a, "lu")?;
    let (m, n) = a.shape();
    if a.is_empty() {
        return Ok(Lu {
            l: NdArray::zeros(m, m.min(n)),
            u: NdArray::zeros(m.min(n), n),
            perm: (0..m).collect(),
        });
    }
    let f = a.as_faer().partial_piv_lu();
    let (l, u) = (NdArray::from_faer(f.L()), NdArray::from_faer(f.U()));
    let perm = f.P().arrays().0.to_vec();
    // Normaliza para L com diagonal unitária, independentemente da convenção do backend.
    let k = m.min(n);
    // Normaliza para L com diagonal unitária independentemente da convenção do backend
    // (com o faer atual, d = 1 e a operação é a identidade).
    let d: Vec<T> = (0..k).map(|i| l.get(i, i)).collect();
    let l2 = NdArray::from_fn(l.rows(), l.cols(), |i, j| l.get(i, j) / d[j]);
    let u2 = NdArray::from_fn(u.rows(), u.cols(), |i, j| u.get(i, j) * d[i]);
    Ok(Lu { l: l2, u: u2, perm })
}

/// Fatoração QR completa (Householder).
pub fn qr<T: LinScalar>(a: &NdArray<T>) -> Result<Qr<T>, LinalgError> {
    require_finite(a, "qr")?;
    let (m, n) = a.shape();
    if a.is_empty() {
        return Ok(Qr {
            q: NdArray::identity(m),
            r: NdArray::zeros(m, n),
        });
    }
    let f = a.as_faer().qr();
    let q = NdArray::from_faer(f.compute_Q().as_ref());
    let r_raw = f.R();
    let r = NdArray::from_fn(m, n, |i, j| {
        if i <= j && i < r_raw.nrows() {
            r_raw[(i, j)]
        } else {
            T::zero()
        }
    });
    Ok(Qr { q, r })
}

/// Fatoração de Cholesky `A = Rᴴ·R` com `R` triangular superior.
///
/// Usa apenas o triângulo superior de `A`, que é tratada como hermitiana.
pub fn chol<T: LinScalar>(a: &NdArray<T>) -> Result<Chol<T>, LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "chol",
            shape: a.shape(),
        });
    }
    require_finite(a, "chol")?;
    if a.is_empty() {
        return Ok(Chol {
            r: NdArray::zeros(0, 0),
        });
    }
    // O backend lê o triângulo inferior; espelhamos o superior (conjugado) para respeitar a
    // convenção acima. A diagonal de uma hermitiana é real.
    let n = a.rows();
    let sym = NdArray::from_fn(n, n, |i, j| match i.cmp(&j) {
        std::cmp::Ordering::Greater => a.get(j, i).conj(),
        std::cmp::Ordering::Equal => T::from_f64(a.get(i, i).re()),
        std::cmp::Ordering::Less => a.get(i, j),
    });
    let llt = sym
        .as_faer()
        .llt(Side::Lower)
        .map_err(|_| LinalgError::NotPositiveDefinite)?;
    let l = llt.L();
    Ok(Chol {
        r: NdArray::from_fn(n, n, |i, j| {
            if i <= j {
                Scalar::conj(l[(j, i)])
            } else {
                T::zero()
            }
        }),
    })
}

/// Decomposição em valores singulares. Com `economy = true`, devolve apenas as `min(m, n)`
/// primeiras colunas de `U` e `V`.
pub fn svd<T: LinScalar>(a: &NdArray<T>, economy: bool) -> Result<Svd<T>, LinalgError> {
    require_finite(a, "svd")?;
    let (m, n) = a.shape();
    if a.is_empty() {
        let (um, vn) = if economy { (0, 0) } else { (m, n) };
        return Ok(Svd {
            u: NdArray::identity(m).submatrix(0, m, 0, um),
            s: vec![],
            v: NdArray::identity(n).submatrix(0, n, 0, vn),
        });
    }
    let f = if economy {
        a.as_faer().thin_svd()
    } else {
        a.as_faer().svd()
    }
    .map_err(|_| LinalgError::NoConvergence { operation: "svd" })?;
    let sv = f.S().column_vector();
    Ok(Svd {
        u: NdArray::from_faer(f.U()),
        s: (0..sv.nrows()).map(|i| Scalar::re(sv[i])).collect(),
        v: NdArray::from_faer(f.V()),
    })
}

/// Valores singulares em ordem não crescente.
pub fn singular_values<T: LinScalar>(a: &NdArray<T>) -> Result<Vec<f64>, LinalgError> {
    require_finite(a, "svd")?;
    if a.is_empty() {
        return Ok(vec![]);
    }
    let mut s = a
        .as_faer()
        .singular_values()
        .map_err(|_| LinalgError::NoConvergence { operation: "svd" })?;
    s.sort_by(|x, y| y.total_cmp(x));
    Ok(s)
}

/// Autovalores e autovetores. Matrizes exatamente hermitianas (simétricas, se reais) usam o
/// algoritmo hermitiano (autovalores reais, vetores ortonormais); as demais, o algoritmo geral
/// (resultado complexo), precedido de balanceamento ([`balance`](super::balance)).
/// Para desligar o balanceamento ou escolher a ordem, use [`eig_with`](super::eig_with).
pub fn eig<T: LinScalar>(a: &NdArray<T>) -> Result<Eig<T>, LinalgError> {
    super::eig_with(a, super::EigOptions::default())
}

/// [`eig`] sem balanceamento e sem ordenação.
pub(crate) fn eig_unbalanced<T: LinScalar>(a: &NdArray<T>) -> Result<Eig<T>, LinalgError> {
    if !a.is_square() {
        return Err(LinalgError::NotSquare {
            operation: "eig",
            shape: a.shape(),
        });
    }
    require_finite(a, "eig")?;
    if a.is_empty() {
        return Ok(Eig::Symmetric {
            values: vec![],
            vectors: NdArray::zeros(0, 0),
        });
    }
    if a.is_hermitian() {
        let f = a
            .as_faer()
            .self_adjoint_eigen(Side::Lower)
            .map_err(|_| LinalgError::NoConvergence { operation: "eig" })?;
        let s = f.S().column_vector();
        return Ok(Eig::Symmetric {
            values: (0..s.nrows()).map(|i| Scalar::re(s[i])).collect(),
            vectors: NdArray::from_faer(f.U()),
        });
    }
    let f = a
        .as_faer()
        .eigen()
        .map_err(|_| LinalgError::NoConvergence { operation: "eig" })?;
    let s = f.S().column_vector();
    Ok(Eig::General {
        values: (0..s.nrows()).map(|i| s[i]).collect(),
        vectors: ComplexMatrix::from_faer(f.U()),
    })
}
