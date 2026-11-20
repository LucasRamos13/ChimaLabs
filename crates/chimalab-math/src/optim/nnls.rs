//! Mínimos quadrados não negativos: min ‖A x − b‖₂ com x ≥ 0, pelo método de conjunto ativo de
//! C. L. Lawson & R. J. Hanson, *Solving Least Squares Problems* (1974), cap. 23.

use crate::linalg::{Matrix, solve};

use super::{ExitFlag, OptimError, Options, Procedure, Tracker};

/// Resultado de [`nnls`].
#[derive(Clone, Debug, PartialEq)]
pub struct NnlsResult {
    /// Solução (x ≥ 0).
    pub x: Vec<f64>,
    /// ‖b − A x‖₂².
    pub resnorm: f64,
    /// Resíduo b − A x.
    pub residual: Vec<f64>,
    /// Multiplicadores w = Aᵀ(b − A x): w ≤ tol onde x = 0 e w ≈ 0 onde x > 0 (KKT).
    pub dual: Vec<f64>,
    /// Motivo do término.
    pub exit: ExitFlag,
    /// Iterações (variáveis que entraram no conjunto livre).
    pub iterations: usize,
    /// Histórico, se pedido.
    pub history: Vec<super::Iteration>,
}

/// Mínimos quadrados com x ≥ 0. `a` é m×n; `b` tem m elementos.
///
/// Tolerância dos multiplicadores: `tol_x` (padrão 10·ε_máq·‖A‖₁·max(m, n)·‖b‖∞, a escala dos
/// erros de arredondamento de w). Padrão de `max_iter` = 3n; cada iteração resolve um
/// subproblema de mínimos quadrados por QR com pivotamento de colunas.
///
/// Erros: `formaIncompativel` (b com tamanho ≠ m), `opcaoInvalida` (A ou b não finitos),
/// `toleranciaInvalida`.
pub fn nnls(a: &Matrix, b: &[f64], opts: &mut Options<'_>) -> Result<NnlsResult, OptimError> {
    let (m, n) = a.shape();
    if b.len() != m {
        return Err(OptimError::ShapeMismatch {
            expected: m,
            actual: b.len(),
        });
    }
    if a.as_slice().iter().chain(b).any(|v| !v.is_finite()) {
        return Err(OptimError::InvalidOption("A e b devem ser finitos"));
    }
    let norm_a = (0..n)
        .map(|j| (0..m).map(|i| a.get(i, j).abs()).sum::<f64>())
        .fold(0.0f64, f64::max);
    let norm_b = b.iter().fold(0.0f64, |s, v| s.max(v.abs()));
    let tol = opts
        .tol_x
        .unwrap_or(10.0 * f64::EPSILON * norm_a * (m.max(n) as f64) * norm_b.max(f64::MIN_POSITIVE));
    let mut t = Tracker::new(opts, 3 * n.max(1), usize::MAX)?;
    let mut x = vec![0.0; n];
    let mut free = vec![false; n];
    let residual = |x: &[f64]| -> Vec<f64> {
        (0..m)
            .map(|i| b[i] - (0..n).map(|j| a.get(i, j) * x[j]).sum::<f64>())
            .collect()
    };
    let dual = |r: &[f64]| -> Vec<f64> {
        (0..n)
            .map(|j| (0..m).map(|i| a.get(i, j) * r[i]).sum::<f64>())
            .collect()
    };
    let resnorm = |r: &[f64]| r.iter().map(|v| v * v).sum::<f64>();
    let mut r = residual(&x);
    let mut w = dual(&r);
    let mut exit = t.report(&x, resnorm(&r), Procedure::Initial, None);
    while exit.is_none() {
        // Variável fixa (em zero) com o maior multiplicador positivo.
        let entering = (0..n)
            .filter(|&j| !free[j] && w[j] > tol)
            .max_by(|&i, &j| w[i].total_cmp(&w[j]));
        let Some(jmax) = entering else {
            exit = Some(ExitFlag::Converged);
            break;
        };
        free[jmax] = true;
        let mut first = true;
        let mut rejected = false;
        loop {
            let z = solve_free(a, b, &free);
            // A variável que entrou sai com z ≤ 0 já no primeiro subproblema: o multiplicador
            // positivo era arredondamento. Fica fora até a próxima mudança do conjunto
            // (Lawson & Hanson, §23.3, passo 6), o que evita ciclo.
            if first && z[jmax] <= 0.0 {
                free[jmax] = false;
                w[jmax] = 0.0;
                rejected = true;
                break;
            }
            first = false;
            if (0..n).filter(|&j| free[j]).all(|j| z[j] > 0.0) {
                x = z;
                break;
            }
            // Passo até a fronteira: α = min xⱼ/(xⱼ − zⱼ) sobre as livres com zⱼ ≤ 0.
            let (alpha, jblock) = (0..n)
                .filter(|&j| free[j] && z[j] <= 0.0)
                .map(|j| (x[j] / (x[j] - z[j]), j))
                .fold((f64::INFINITY, n), |acc, v| if v.0 < acc.0 { v } else { acc });
            for j in (0..n).filter(|&j| free[j]) {
                x[j] += alpha * (z[j] - x[j]);
            }
            x[jblock] = 0.0;
            free[jblock] = false;
            for j in 0..n {
                if free[j] && x[j] <= 0.0 {
                    x[j] = 0.0;
                    free[j] = false;
                }
            }
            t.report(&x, resnorm(&residual(&x)), Procedure::Remove, None);
        }
        if rejected {
            continue;
        }
        r = residual(&x);
        w = dual(&r);
        t.iterations += 1;
        exit = t.report(&x, resnorm(&r), Procedure::Add, None);
    }
    let exit = exit.unwrap_or(ExitFlag::Converged);
    let iterations = t.iterations;
    let res = t.finish((), 0.0, exit, None);
    Ok(NnlsResult {
        resnorm: resnorm(&r),
        residual: r,
        dual: w,
        x,
        exit,
        iterations,
        history: res.history,
    })
}

/// Solução de mínimos quadrados com as variáveis livres; as fixas valem 0.
fn solve_free(a: &Matrix, b: &[f64], free: &[bool]) -> Vec<f64> {
    let cols: Vec<usize> = (0..free.len()).filter(|&j| free[j]).collect();
    let m = a.rows();
    let sub = Matrix::from_fn(m, cols.len(), |i, k| a.get(i, cols[k]));
    let rhs = Matrix::column(b);
    let mut s = vec![0.0; free.len()];
    if let Ok(sol) = solve(&sub, &rhs) {
        for (k, &j) in cols.iter().enumerate() {
            s[j] = sol.x.get(k, 0);
        }
    }
    s
}
