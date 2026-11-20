//! Precondicionadores: Jacobi, ILU(0) e IC(0) no padrão de A (Saad, *Iterative Methods for
//! Sparse Linear Systems*, 2.ª ed., 2003, §10.2–10.3).

use super::{SparseError, SparseMatrix};

/// Precondicionador M ≈ A (os iterativos aplicam M⁻¹).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preconditioner {
    /// Sem precondicionamento (padrão).
    #[default]
    None,
    /// Diagonal de A.
    Jacobi,
    /// Cholesky incompleta sem preenchimento (A simétrica definida positiva). Para A simétrica,
    /// é a ILU(0) com pivôs positivos: U = D·Lᵀ.
    Ic0,
    /// LU incompleta sem preenchimento.
    Ilu0,
}

/// Precondicionador construído.
pub(crate) enum Built {
    Identity,
    Diag(Vec<f64>),
    /// Fatores em CSR: L (diagonal unitária implícita, só a parte estrita) e U (com diagonal).
    Lu {
        row_ptr: Vec<usize>,
        col: Vec<usize>,
        val: Vec<f64>,
        diag_pos: Vec<usize>,
    },
}

impl Preconditioner {
    pub(crate) fn build(self, a: &SparseMatrix) -> Result<Built, SparseError> {
        let n = a.shape().0;
        match self {
            Self::None => Ok(Built::Identity),
            Self::Jacobi => {
                let d = a.diagonal();
                if d.contains(&0.0) {
                    return Err(SparseError::Singular);
                }
                Ok(Built::Diag(d.iter().map(|v| 1.0 / v).collect()))
            }
            Self::Ic0 | Self::Ilu0 => {
                // CSR de A = CSC de Aᵀ.
                let at = a.transpose();
                let row_ptr = at.col_ptr().to_vec();
                let col = at.row_idx().to_vec();
                let mut val = at.values().to_vec();
                let mut diag_pos = vec![usize::MAX; n];
                for (i, dp) in diag_pos.iter_mut().enumerate() {
                    let linha = row_ptr[i]..row_ptr[i + 1];
                    *dp = linha
                        .clone()
                        .find(|&k| col[k] == i)
                        .ok_or(SparseError::Singular)?;
                }
                // ILU(0), variante IKJ: só atualiza posições existentes no padrão.
                let mut pos = vec![usize::MAX; n];
                for i in 0..n {
                    for k in row_ptr[i]..row_ptr[i + 1] {
                        pos[col[k]] = k;
                    }
                    for kk in row_ptr[i]..row_ptr[i + 1] {
                        let k = col[kk];
                        if k >= i {
                            break;
                        }
                        let piv = val[diag_pos[k]];
                        val[kk] /= piv;
                        let lik = val[kk];
                        for kj in diag_pos[k] + 1..row_ptr[k + 1] {
                            let j = col[kj];
                            if pos[j] != usize::MAX {
                                val[pos[j]] -= lik * val[kj];
                            }
                        }
                    }
                    let d = val[diag_pos[i]];
                    if d == 0.0 || !d.is_finite() || (self == Self::Ic0 && d < 0.0) {
                        return Err(SparseError::Singular);
                    }
                    for k in row_ptr[i]..row_ptr[i + 1] {
                        pos[col[k]] = usize::MAX;
                    }
                }
                Ok(Built::Lu {
                    row_ptr,
                    col,
                    val,
                    diag_pos,
                })
            }
        }
    }
}

impl Built {
    /// z = M⁻¹·r.
    pub(crate) fn apply(&self, r: &[f64], z: &mut [f64]) {
        match self {
            Self::Identity => z.copy_from_slice(r),
            Self::Diag(d) => {
                for ((zi, ri), di) in z.iter_mut().zip(r).zip(d) {
                    *zi = ri * di;
                }
            }
            Self::Lu {
                row_ptr,
                col,
                val,
                diag_pos,
            } => {
                let n = r.len();
                // L·y = r (L com diagonal unitária, entradas à esquerda da diagonal).
                for i in 0..n {
                    let mut s = r[i];
                    for k in row_ptr[i]..diag_pos[i] {
                        s -= val[k] * z[col[k]];
                    }
                    z[i] = s;
                }
                // U·z = y.
                for i in (0..n).rev() {
                    let mut s = z[i];
                    for k in diag_pos[i] + 1..row_ptr[i + 1] {
                        s -= val[k] * z[col[k]];
                    }
                    z[i] = s / val[diag_pos[i]];
                }
            }
        }
    }
}
