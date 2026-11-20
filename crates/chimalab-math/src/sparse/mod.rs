//! Matrizes esparsas e solvers (funcionalidade F041, RF-041; DT-30).
//!
//! [`SparseMatrix`] guarda o formato CSC (colunas comprimidas): para cada coluna j, os índices de
//! linha em ordem crescente e os valores, de `col_ptr[j]` a `col_ptr[j + 1]`. A memória é
//! proporcional ao número de não nulos (nnz) mais o número de colunas.
//!
//! - Resolução direta: [`spsolve`] (Cholesky supernodal ou LU com ordenação, pelo `faer`).
//! - Iterativos: [`pcg`], [`gmres`], [`bicgstab`], [`minres`], com [`Preconditioner`].
//! - Autovalores parciais de simétricas: [`eigs`] (Krylov–Schur, deslocamento-inversão).

mod eigs;
mod iterative;
mod precond;

use std::fmt;

use faer::linalg::solvers::Solve;
use faer::sparse::{SparseColMat, Triplet};
use faer::{MatMut, Side};

pub use eigs::{EigsOptions, EigsResult, SvdsResult, Which, eigs, svds};
pub use iterative::{IterFlag, IterOptions, IterResult, bicgstab, gmres, lsqr, minres, pcg};
pub use precond::Preconditioner;

use crate::linalg::Matrix;

/// Erros.
#[derive(Clone, Debug, PartialEq)]
pub enum SparseError {
    /// Formas incompatíveis.
    ShapeMismatch {
        /// Esperado.
        expected: (usize, usize),
        /// Recebido.
        actual: (usize, usize),
    },
    /// Índice fora da forma.
    IndexOutOfBounds {
        /// Linha.
        row: usize,
        /// Coluna.
        col: usize,
    },
    /// Valor não finito.
    NonFinite,
    /// Matriz singular (pivô nulo na fatoração).
    Singular,
    /// Matriz não quadrada onde é exigida.
    NotSquare((usize, usize)),
    /// Opção inválida.
    InvalidOption(&'static str),
}

impl SparseError {
    /// Identificador estável.
    pub fn id(&self) -> &'static str {
        match self {
            Self::ShapeMismatch { .. } => "chimalab:sparse:formaIncompativel",
            Self::IndexOutOfBounds { .. } => "chimalab:sparse:indiceForaDaForma",
            Self::NonFinite => "chimalab:sparse:naoFinito",
            Self::Singular => "chimalab:sparse:singular",
            Self::NotSquare(_) => "chimalab:sparse:naoQuadrada",
            Self::InvalidOption(_) => "chimalab:sparse:opcaoInvalida",
        }
    }
}

impl fmt::Display for SparseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShapeMismatch { expected, actual } => write!(f, "forma {actual:?}; esperada {expected:?}"),
            Self::IndexOutOfBounds { row, col } => write!(f, "índice ({row}, {col}) fora da forma"),
            Self::NonFinite => write!(f, "valor não finito"),
            Self::Singular => write!(f, "matriz singular"),
            Self::NotSquare(s) => write!(f, "matriz {s:?} não é quadrada"),
            Self::InvalidOption(m) => write!(f, "opção inválida: {m}"),
        }
    }
}

impl std::error::Error for SparseError {}

/// Matriz esparsa real em CSC.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseMatrix {
    nrows: usize,
    ncols: usize,
    col_ptr: Vec<usize>,
    row_idx: Vec<usize>,
    values: Vec<f64>,
}

impl SparseMatrix {
    /// A partir de triplas (i, j, v): repetidos são somados e zeros (inclusive os que resultam da
    /// soma) não são guardados.
    ///
    /// Erros: `indiceForaDaForma`.
    pub fn from_triplets(
        nrows: usize,
        ncols: usize,
        triplets: &[(usize, usize, f64)],
    ) -> Result<Self, SparseError> {
        if let Some(&(i, j, _)) = triplets.iter().find(|(i, j, _)| *i >= nrows || *j >= ncols) {
            return Err(SparseError::IndexOutOfBounds { row: i, col: j });
        }
        let mut t: Vec<(usize, usize, f64)> = triplets.to_vec();
        t.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        let mut col_ptr = vec![0usize; ncols + 1];
        let mut row_idx = Vec::with_capacity(t.len());
        let mut values = Vec::with_capacity(t.len());
        let mut k = 0;
        while k < t.len() {
            let (i, j, mut v) = t[k];
            k += 1;
            while k < t.len() && t[k].0 == i && t[k].1 == j {
                v += t[k].2;
                k += 1;
            }
            if v != 0.0 {
                row_idx.push(i);
                values.push(v);
                col_ptr[j + 1] += 1;
            }
        }
        for j in 0..ncols {
            col_ptr[j + 1] += col_ptr[j];
        }
        Ok(Self {
            nrows,
            ncols,
            col_ptr,
            row_idx,
            values,
        })
    }

    /// A partir de uma matriz densa (zeros não são guardados).
    pub fn from_dense(a: &Matrix) -> Self {
        let (m, n) = a.shape();
        let mut t = Vec::new();
        for j in 0..n {
            for i in 0..m {
                let v = a.get(i, j);
                if v != 0.0 {
                    t.push((i, j, v));
                }
            }
        }
        Self::from_triplets(m, n, &t).expect("índices dentro da forma")
    }

    /// Identidade n×n.
    pub fn identity(n: usize) -> Self {
        Self {
            nrows: n,
            ncols: n,
            col_ptr: (0..=n).collect(),
            row_idx: (0..n).collect(),
            values: vec![1.0; n],
        }
    }

    /// Forma (linhas, colunas).
    pub fn shape(&self) -> (usize, usize) {
        (self.nrows, self.ncols)
    }

    /// Número de não nulos guardados.
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    /// Ponteiros de coluna (ncols + 1).
    pub fn col_ptr(&self) -> &[usize] {
        &self.col_ptr
    }

    /// Índices de linha (nnz).
    pub fn row_idx(&self) -> &[usize] {
        &self.row_idx
    }

    /// Valores (nnz).
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Elemento (i, j) (0 se não guardado). Busca binária na coluna.
    ///
    /// # Panics
    /// Fora da forma.
    pub fn get(&self, i: usize, j: usize) -> f64 {
        assert!(i < self.nrows && j < self.ncols, "índice fora da forma");
        let (a, b) = (self.col_ptr[j], self.col_ptr[j + 1]);
        match self.row_idx[a..b].binary_search(&i) {
            Ok(k) => self.values[a + k],
            Err(_) => 0.0,
        }
    }

    /// Triplas (i, j, v) em ordem de coluna.
    pub fn triplets(&self) -> Vec<(usize, usize, f64)> {
        (0..self.ncols)
            .flat_map(|j| (self.col_ptr[j]..self.col_ptr[j + 1]).map(move |k| (j, k)))
            .map(|(j, k)| (self.row_idx[k], j, self.values[k]))
            .collect()
    }

    /// Matriz densa.
    pub fn to_dense(&self) -> Matrix {
        let mut m = Matrix::zeros(self.nrows, self.ncols);
        for (i, j, v) in self.triplets() {
            m.set(i, j, v);
        }
        m
    }

    /// Transposta.
    pub fn transpose(&self) -> Self {
        let t: Vec<(usize, usize, f64)> = self.triplets().into_iter().map(|(i, j, v)| (j, i, v)).collect();
        Self::from_triplets(self.ncols, self.nrows, &t).expect("índices válidos")
    }

    /// Diagonal principal (min(m, n) valores).
    pub fn diagonal(&self) -> Vec<f64> {
        (0..self.nrows.min(self.ncols)).map(|i| self.get(i, i)).collect()
    }

    /// y = A·x.
    ///
    /// Erros: `formaIncompativel`.
    pub fn mul_vec(&self, x: &[f64]) -> Result<Vec<f64>, SparseError> {
        if x.len() != self.ncols {
            return Err(SparseError::ShapeMismatch {
                expected: (self.ncols, 1),
                actual: (x.len(), 1),
            });
        }
        let mut y = vec![0.0; self.nrows];
        self.mul_vec_into(x, &mut y);
        Ok(y)
    }

    pub(crate) fn mul_vec_into(&self, x: &[f64], y: &mut [f64]) {
        y.iter_mut().for_each(|v| *v = 0.0);
        for j in 0..self.ncols {
            let xj = x[j];
            if xj == 0.0 {
                continue;
            }
            for k in self.col_ptr[j]..self.col_ptr[j + 1] {
                y[self.row_idx[k]] += self.values[k] * xj;
            }
        }
    }

    /// y = Aᵀ·x.
    pub fn mul_vec_t(&self, x: &[f64]) -> Result<Vec<f64>, SparseError> {
        if x.len() != self.nrows {
            return Err(SparseError::ShapeMismatch {
                expected: (self.nrows, 1),
                actual: (x.len(), 1),
            });
        }
        Ok((0..self.ncols)
            .map(|j| {
                (self.col_ptr[j]..self.col_ptr[j + 1])
                    .map(|k| self.values[k] * x[self.row_idx[k]])
                    .sum()
            })
            .collect())
    }

    /// C = A·B esparsa (Gustavson: acumulador denso por coluna, só nos índices tocados).
    pub fn mul(&self, b: &SparseMatrix) -> Result<SparseMatrix, SparseError> {
        if self.ncols != b.nrows {
            return Err(SparseError::ShapeMismatch {
                expected: (self.ncols, b.ncols),
                actual: b.shape(),
            });
        }
        let mut acc = vec![0.0; self.nrows];
        let mut marca = vec![usize::MAX; self.nrows];
        let mut t = Vec::new();
        for j in 0..b.ncols {
            let mut tocados = Vec::new();
            for kb in b.col_ptr[j]..b.col_ptr[j + 1] {
                let (l, bv) = (b.row_idx[kb], b.values[kb]);
                for ka in self.col_ptr[l]..self.col_ptr[l + 1] {
                    let i = self.row_idx[ka];
                    if marca[i] != j {
                        marca[i] = j;
                        acc[i] = 0.0;
                        tocados.push(i);
                    }
                    acc[i] += self.values[ka] * bv;
                }
            }
            for i in tocados {
                t.push((i, j, acc[i]));
            }
        }
        SparseMatrix::from_triplets(self.nrows, b.ncols, &t)
    }

    /// C = α·A + β·B.
    pub fn add_scaled(&self, alpha: f64, b: &SparseMatrix, beta: f64) -> Result<SparseMatrix, SparseError> {
        if self.shape() != b.shape() {
            return Err(SparseError::ShapeMismatch {
                expected: self.shape(),
                actual: b.shape(),
            });
        }
        let mut t: Vec<(usize, usize, f64)> = self
            .triplets()
            .into_iter()
            .map(|(i, j, v)| (i, j, alpha * v))
            .collect();
        t.extend(b.triplets().into_iter().map(|(i, j, v)| (i, j, beta * v)));
        SparseMatrix::from_triplets(self.nrows, self.ncols, &t)
    }

    /// Simétrica (exatamente, guardados e valores).
    pub fn is_symmetric(&self) -> bool {
        self.nrows == self.ncols && *self == self.transpose()
    }

    fn to_faer(&self) -> SparseColMat<usize, f64> {
        let t: Vec<Triplet<usize, usize, f64>> = self
            .triplets()
            .into_iter()
            .map(|(i, j, v)| Triplet::new(i, j, v))
            .collect();
        SparseColMat::try_new_from_triplets(self.nrows, self.ncols, &t).expect("triplas válidas")
    }
}

/// Método usado por [`spsolve`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectMethod {
    /// Cholesky supernodal (simétrica definida positiva).
    Cholesky,
    /// LU com pivotamento parcial e ordenação de colunas.
    Lu,
}

/// Resolve A·x = b diretamente. Tenta Cholesky se A é simétrica com diagonal positiva; se a
/// fatoração falhar (não definida positiva), usa LU.
///
/// Erros: `naoQuadrada`, `formaIncompativel`, `naoFinito` (A ou b), `singular`.
pub fn spsolve(a: &SparseMatrix, b: &[f64]) -> Result<(Vec<f64>, DirectMethod), SparseError> {
    let (m, n) = a.shape();
    if m != n {
        return Err(SparseError::NotSquare((m, n)));
    }
    if b.len() != n {
        return Err(SparseError::ShapeMismatch {
            expected: (n, 1),
            actual: (b.len(), 1),
        });
    }
    if a.values.iter().chain(b).any(|v| !v.is_finite()) {
        return Err(SparseError::NonFinite);
    }
    if n == 0 {
        return Ok((Vec::new(), DirectMethod::Lu));
    }
    let fa = a.to_faer();
    let mut x = b.to_vec();
    if a.diagonal().iter().all(|d| *d > 0.0)
        && a.is_symmetric()
        && let Ok(llt) = fa.sp_cholesky(Side::Lower)
    {
        llt.solve_in_place(MatMut::from_column_major_slice_mut(&mut x, n, 1));
        if x.iter().all(|v| v.is_finite()) {
            return Ok((x, DirectMethod::Cholesky));
        }
        x.copy_from_slice(b);
    }
    let lu = fa.sp_lu().map_err(|_| SparseError::Singular)?;
    lu.solve_in_place(MatMut::from_column_major_slice_mut(&mut x, n, 1));
    if x.iter().any(|v| !v.is_finite()) {
        return Err(SparseError::Singular);
    }
    Ok((x, DirectMethod::Lu))
}

/// Ordenação de grau mínimo aproximado (AMD; P. R. Amestoy, T. A. Davis & I. S. Duff, SIAM J.
/// Matrix Anal. Appl. 17, 1996) do padrão de A + Aᵀ, para reduzir o preenchimento de Cholesky
/// ou LU simétrica: `p[k]` é a linha/coluna original na posição k. Implementação de faer.
///
/// Erros: `naoQuadrada`.
pub fn amd(a: &SparseMatrix) -> Result<Vec<usize>, SparseError> {
    use faer::dyn_stack::{MemBuffer, MemStack};
    use faer::sparse::linalg::amd as famd;
    let (m, n) = a.shape();
    if m != n {
        return Err(SparseError::NotSquare((m, n)));
    }
    let f = a.to_faer();
    let mut perm = vec![0usize; n];
    let mut inv = vec![0usize; n];
    let mut buf = MemBuffer::new(famd::order_maybe_unsorted_scratch::<usize>(n, a.nnz().max(1)));
    famd::order_maybe_unsorted(
        &mut perm,
        &mut inv,
        f.symbolic(),
        famd::Control::default(),
        MemStack::new(&mut buf),
    )
    .expect("padrão válido");
    Ok(perm)
}

/// Ordenação de colunas por grau mínimo aproximado (COLAMD; T. A. Davis, J. R. Gilbert, S. I.
/// Larimore & E. G. Ng, ACM TOMS 30, 2004) para LU com pivotamento parcial ou QR de A (m × n
/// qualquer): `q[k]` é a coluna original na posição k. Implementação de faer.
pub fn colamd(a: &SparseMatrix) -> Vec<usize> {
    use faer::dyn_stack::{MemBuffer, MemStack};
    use faer::sparse::linalg::colamd as fcol;
    let (m, n) = a.shape();
    let f = a.to_faer();
    let mut perm = vec![0usize; n];
    let mut inv = vec![0usize; n];
    let mut buf = MemBuffer::new(fcol::order_scratch::<usize>(m, n, a.nnz().max(1)));
    fcol::order(
        &mut perm,
        &mut inv,
        f.symbolic(),
        fcol::Control::default(),
        MemStack::new(&mut buf),
    )
    .expect("padrão válido");
    perm
}
