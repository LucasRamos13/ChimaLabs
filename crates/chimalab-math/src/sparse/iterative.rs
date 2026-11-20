//! Métodos iterativos de Krylov.
//!
//! Fontes:
//! - M. R. Hestenes & E. Stiefel, J. Res. NBS 49 (1952) (gradiente conjugado);
//! - Y. Saad & M. H. Schultz, SIAM J. Sci. Stat. Comput. 7 (1986) (GMRES, com rotações de Givens);
//! - H. A. van der Vorst, SIAM J. Sci. Stat. Comput. 13 (1992) (BiCGSTAB);
//! - C. C. Paige & M. A. Saunders, SIAM J. Numer. Anal. 12 (1975) (MINRES).
//!
//! A convergência é declarada pelo resíduo verdadeiro ‖b − A·x‖/‖b‖ ≤ tol. Quando a recorrência
//! indica convergência, o resíduo é recalculado e, se ainda não basta, a iteração continua a
//! partir dele (substituição do resíduo).

use super::precond::Built;
use super::{Preconditioner, SparseError, SparseMatrix};

/// Opções dos iterativos.
#[derive(Clone, Debug, PartialEq)]
pub struct IterOptions {
    /// Tolerância do resíduo relativo (padrão 1e-8).
    pub tol: f64,
    /// Máximo de iterações (padrão max(n, 100); no GMRES, total de iterações internas).
    pub max_iter: Option<usize>,
    /// GMRES: dimensão do subespaço antes do reinício (padrão min(n, 30)).
    pub restart: Option<usize>,
    /// Chute inicial (padrão zero).
    pub x0: Option<Vec<f64>>,
    /// Precondicionador.
    pub precond: Preconditioner,
}

impl Default for IterOptions {
    fn default() -> Self {
        Self {
            tol: 1e-8,
            max_iter: None,
            restart: None,
            x0: None,
            precond: Preconditioner::None,
        }
    }
}

/// Motivo do término.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterFlag {
    /// Resíduo relativo ≤ tol.
    Converged,
    /// Atingiu o máximo de iterações (o melhor iterado é devolvido).
    MaxIterations,
    /// Divisão por (quase) zero na recorrência: o método não pode continuar (ex.: CG em matriz
    /// não definida positiva).
    Breakdown,
    /// Valores não finitos.
    NonFinite,
}

impl IterFlag {
    /// Identificador estável do aviso (None se convergiu).
    pub fn warning_id(self) -> Option<&'static str> {
        match self {
            Self::Converged => None,
            Self::MaxIterations => Some("chimalab:sparse:naoConvergiu"),
            Self::Breakdown => Some("chimalab:sparse:colapso"),
            Self::NonFinite => Some("chimalab:sparse:naoFinito"),
        }
    }
}

/// Resultado.
#[derive(Clone, Debug, PartialEq)]
pub struct IterResult {
    /// Solução aproximada.
    pub x: Vec<f64>,
    /// Motivo do término.
    pub flag: IterFlag,
    /// Resíduo relativo verdadeiro ‖b − A·x‖/‖b‖ (0 se b = 0).
    pub relres: f64,
    /// Iterações.
    pub iterations: usize,
    /// Norma do resíduo da recorrência a cada iteração (a primeira é ‖r₀‖).
    pub resvec: Vec<f64>,
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn norm(a: &[f64]) -> f64 {
    let m = a.iter().fold(0.0f64, |s, v| s.max(v.abs()));
    if m == 0.0 || !m.is_finite() {
        return m;
    }
    m * a.iter().map(|v| (v / m).powi(2)).sum::<f64>().sqrt()
}

struct Setup {
    n: usize,
    x: Vec<f64>,
    nb: f64,
    max_iter: usize,
    m: Built,
}

fn setup(a: &SparseMatrix, b: &[f64], o: &IterOptions) -> Result<Setup, SparseError> {
    let (r, c) = a.shape();
    if r != c {
        return Err(SparseError::NotSquare((r, c)));
    }
    if b.len() != r {
        return Err(SparseError::ShapeMismatch {
            expected: (r, 1),
            actual: (b.len(), 1),
        });
    }
    if !(o.tol.is_finite() && o.tol > 0.0) {
        return Err(SparseError::InvalidOption("tol finita e > 0"));
    }
    let x = match &o.x0 {
        Some(x0) if x0.len() != r => {
            return Err(SparseError::ShapeMismatch {
                expected: (r, 1),
                actual: (x0.len(), 1),
            });
        }
        Some(x0) => x0.clone(),
        None => vec![0.0; r],
    };
    if a.values().iter().chain(b).chain(&x).any(|v| !v.is_finite()) {
        return Err(SparseError::NonFinite);
    }
    Ok(Setup {
        n: r,
        x,
        nb: norm(b),
        max_iter: o.max_iter.unwrap_or(r.max(100)),
        m: o.precond.build(a)?,
    })
}

fn true_residual(a: &SparseMatrix, b: &[f64], x: &[f64], r: &mut [f64]) -> f64 {
    a.mul_vec_into(x, r);
    for (ri, bi) in r.iter_mut().zip(b) {
        *ri = bi - *ri;
    }
    norm(r)
}

fn finish(
    a: &SparseMatrix,
    b: &[f64],
    x: Vec<f64>,
    flag: IterFlag,
    iterations: usize,
    resvec: Vec<f64>,
    nb: f64,
) -> IterResult {
    let mut r = vec![0.0; b.len()];
    let nr = true_residual(a, b, &x, &mut r);
    let relres = if nb == 0.0 { nr } else { nr / nb };
    let flag = if x.iter().any(|v| !v.is_finite()) {
        IterFlag::NonFinite
    } else {
        flag
    };
    IterResult {
        x,
        flag,
        relres,
        iterations,
        resvec,
    }
}

/// Melhor iterado para métodos de resíduo não monótono (PCG, BiCGSTAB): guarda o x de menor
/// resíduo da recorrência e, sem convergência, devolve o de menor resíduo verdadeiro entre ele e
/// o último. Regressão: o BiCGSTAB divergente devolvia o último iterado (relres ~1e23, pior que
/// x₀), embora a documentação prometesse o melhor.
struct Melhor {
    nr: f64,
    x: Vec<f64>,
}

impl Melhor {
    fn observa(&mut self, nr: f64, x: &[f64]) {
        if nr < self.nr {
            self.nr = nr;
            self.x.copy_from_slice(x);
        }
    }

    /// (x, flag) final. Se o último iterado estourou, a flag é `NonFinite`, mas o x devolvido é
    /// o melhor finito.
    fn escolhe(self, a: &SparseMatrix, b: &[f64], x: Vec<f64>, flag: IterFlag) -> (Vec<f64>, IterFlag) {
        if flag == IterFlag::Converged {
            return (x, flag);
        }
        let flag = if x.iter().all(|v| v.is_finite()) {
            flag
        } else {
            IterFlag::NonFinite
        };
        let mut r = vec![0.0; b.len()];
        let ultimo = true_residual(a, b, &x, &mut r);
        let melhor = true_residual(a, b, &self.x, &mut r);
        // NaN no último (divergência) perde para qualquer valor finito.
        if melhor < ultimo || (ultimo.is_nan() && !melhor.is_nan()) {
            (self.x, flag)
        } else {
            (x, flag)
        }
    }
}

/// Gradiente conjugado precondicionado, para A simétrica definida positiva (M também SPD).
///
/// Erros: `naoQuadrada`, `formaIncompativel`, `naoFinito`, `opcaoInvalida`, `singular` (o
/// precondicionador não pôde ser construído).
pub fn pcg(a: &SparseMatrix, b: &[f64], opts: &IterOptions) -> Result<IterResult, SparseError> {
    let Setup {
        n,
        mut x,
        nb,
        max_iter,
        m,
    } = setup(a, b, opts)?;
    let mut r = vec![0.0; n];
    let mut nr = true_residual(a, b, &x, &mut r);
    let mut resvec = vec![nr];
    if nb == 0.0 {
        return Ok(finish(a, b, vec![0.0; n], IterFlag::Converged, 0, resvec, nb));
    }
    let alvo = opts.tol * nb;
    if nr <= alvo {
        return Ok(finish(a, b, x, IterFlag::Converged, 0, resvec, nb));
    }
    let mut z = vec![0.0; n];
    m.apply(&r, &mut z);
    let mut p = z.clone();
    let mut rz = dot(&r, &z);
    let mut q = vec![0.0; n];
    let mut melhor = Melhor { nr, x: x.clone() };
    let mut flag = IterFlag::MaxIterations;
    let mut it = 0;
    while it < max_iter {
        it += 1;
        a.mul_vec_into(&p, &mut q);
        let pq = dot(&p, &q);
        if pq <= 0.0 || !pq.is_finite() || rz <= 0.0 {
            flag = IterFlag::Breakdown;
            break;
        }
        let alpha = rz / pq;
        for i in 0..n {
            x[i] += alpha * p[i];
            r[i] -= alpha * q[i];
        }
        nr = norm(&r);
        resvec.push(nr);
        melhor.observa(nr, &x);
        if nr <= alvo {
            // Substituição do resíduo: confirma com o verdadeiro.
            nr = true_residual(a, b, &x, &mut r);
            if nr <= alvo {
                flag = IterFlag::Converged;
                break;
            }
        }
        m.apply(&r, &mut z);
        let rz_new = dot(&r, &z);
        let beta = rz_new / rz;
        rz = rz_new;
        for i in 0..n {
            p[i] = z[i] + beta * p[i];
        }
    }
    let (x, flag) = melhor.escolhe(a, b, x, flag);
    Ok(finish(a, b, x, flag, it, resvec, nb))
}

/// GMRES(m) com reinício e precondicionamento à direita (o resíduo minimizado é o verdadeiro).
pub fn gmres(a: &SparseMatrix, b: &[f64], opts: &IterOptions) -> Result<IterResult, SparseError> {
    let Setup {
        n,
        mut x,
        nb,
        max_iter,
        m,
    } = setup(a, b, opts)?;
    let restart = opts.restart.unwrap_or(n.min(30)).max(1);
    let mut r = vec![0.0; n];
    let mut nr = true_residual(a, b, &x, &mut r);
    let mut resvec = vec![nr];
    if nb == 0.0 {
        return Ok(finish(a, b, vec![0.0; n], IterFlag::Converged, 0, resvec, nb));
    }
    let alvo = opts.tol * nb;
    let mut flag = IterFlag::MaxIterations;
    let mut it = 0;
    let mut w = vec![0.0; n];
    let mut z = vec![0.0; n];
    'externo: while it < max_iter {
        if nr <= alvo {
            flag = IterFlag::Converged;
            break;
        }
        let mut v: Vec<Vec<f64>> = vec![r.iter().map(|ri| ri / nr).collect()];
        let mut h = vec![vec![0.0; restart]; restart + 1];
        let (mut cs, mut sn) = (vec![0.0; restart], vec![0.0; restart]);
        let mut g = vec![0.0; restart + 1];
        g[0] = nr;
        let mut k = 0;
        while k < restart && it < max_iter {
            it += 1;
            m.apply(&v[k], &mut z);
            a.mul_vec_into(&z, &mut w);
            // Gram–Schmidt modificado, com uma reortogonalização.
            for _ in 0..2 {
                for (j, vj) in v.iter().enumerate() {
                    let hjk = dot(&w, vj);
                    h[j][k] += hjk;
                    for (wi, vi) in w.iter_mut().zip(vj) {
                        *wi -= hjk * vi;
                    }
                }
            }
            let hn = norm(&w);
            h[k + 1][k] = hn;
            // Rotações de Givens anteriores.
            for j in 0..k {
                let t = cs[j] * h[j][k] + sn[j] * h[j + 1][k];
                h[j + 1][k] = -sn[j] * h[j][k] + cs[j] * h[j + 1][k];
                h[j][k] = t;
            }
            let rho = h[k][k].hypot(h[k + 1][k]);
            if rho == 0.0 {
                // A coluna nova de H é nula: o subespaço é invariante e A é singular nele. Usa só
                // as k colunas anteriores (a de agora teria pivô zero; regressão: dava x = ∞).
                flag = IterFlag::Breakdown;
                update(&mut x, &v, &h, &g, k, &m, &mut z);
                break 'externo;
            }
            cs[k] = h[k][k] / rho;
            sn[k] = h[k + 1][k] / rho;
            h[k][k] = rho;
            h[k + 1][k] = 0.0;
            g[k + 1] = -sn[k] * g[k];
            g[k] *= cs[k];
            resvec.push(g[k + 1].abs());
            k += 1;
            if g[k].abs() <= alvo || hn == 0.0 {
                break;
            }
            v.push(w.iter().map(|wi| wi / hn).collect());
        }
        update(&mut x, &v, &h, &g, k, &m, &mut z);
        nr = true_residual(a, b, &x, &mut r);
    }
    if flag == IterFlag::MaxIterations && nr <= alvo {
        flag = IterFlag::Converged;
    }
    Ok(finish(a, b, x, flag, it, resvec, nb))
}

/// x += M⁻¹·V·y, com y da triangular superior H[0..k, 0..k]·y = g[0..k].
fn update(x: &mut [f64], v: &[Vec<f64>], h: &[Vec<f64>], g: &[f64], k: usize, m: &Built, z: &mut [f64]) {
    let mut y = vec![0.0; k];
    for i in (0..k).rev() {
        let s: f64 = (i + 1..k).map(|j| h[i][j] * y[j]).sum();
        y[i] = (g[i] - s) / h[i][i];
    }
    let mut u = vec![0.0; x.len()];
    for (j, yj) in y.iter().enumerate() {
        for (ui, vi) in u.iter_mut().zip(&v[j]) {
            *ui += yj * vi;
        }
    }
    m.apply(&u, z);
    for (xi, zi) in x.iter_mut().zip(z.iter()) {
        *xi += zi;
    }
}

/// BiCGSTAB com precondicionamento à direita, para A não simétrica.
pub fn bicgstab(a: &SparseMatrix, b: &[f64], opts: &IterOptions) -> Result<IterResult, SparseError> {
    let Setup {
        n,
        mut x,
        nb,
        max_iter,
        m,
    } = setup(a, b, opts)?;
    let mut r = vec![0.0; n];
    let mut nr = true_residual(a, b, &x, &mut r);
    let mut resvec = vec![nr];
    if nb == 0.0 {
        return Ok(finish(a, b, vec![0.0; n], IterFlag::Converged, 0, resvec, nb));
    }
    let alvo = opts.tol * nb;
    if nr <= alvo {
        return Ok(finish(a, b, x, IterFlag::Converged, 0, resvec, nb));
    }
    let rhat = r.clone();
    let mut melhor = Melhor { nr, x: x.clone() };
    let (mut rho, mut alpha, mut omega) = (1.0f64, 1.0f64, 1.0f64);
    let mut v = vec![0.0; n];
    let mut p = vec![0.0; n];
    let (mut ph, mut sh, mut s, mut t) = (vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![0.0; n]);
    let mut flag = IterFlag::MaxIterations;
    let mut it = 0;
    while it < max_iter {
        it += 1;
        let rho_new = dot(&rhat, &r);
        if rho_new == 0.0 || omega == 0.0 {
            flag = IterFlag::Breakdown;
            break;
        }
        let beta = (rho_new / rho) * (alpha / omega);
        rho = rho_new;
        for i in 0..n {
            p[i] = r[i] + beta * (p[i] - omega * v[i]);
        }
        m.apply(&p, &mut ph);
        a.mul_vec_into(&ph, &mut v);
        let rv = dot(&rhat, &v);
        if rv == 0.0 {
            flag = IterFlag::Breakdown;
            break;
        }
        alpha = rho / rv;
        for i in 0..n {
            s[i] = r[i] - alpha * v[i];
        }
        if norm(&s) <= alvo {
            for i in 0..n {
                x[i] += alpha * ph[i];
            }
            nr = true_residual(a, b, &x, &mut r);
            resvec.push(nr);
            melhor.observa(nr, &x);
            if nr <= alvo {
                flag = IterFlag::Converged;
                break;
            }
            continue;
        }
        m.apply(&s, &mut sh);
        a.mul_vec_into(&sh, &mut t);
        let tt = dot(&t, &t);
        omega = if tt == 0.0 { 0.0 } else { dot(&t, &s) / tt };
        for i in 0..n {
            x[i] += alpha * ph[i] + omega * sh[i];
            r[i] = s[i] - omega * t[i];
        }
        nr = norm(&r);
        resvec.push(nr);
        melhor.observa(nr, &x);
        if nr <= alvo {
            nr = true_residual(a, b, &x, &mut r);
            if nr <= alvo {
                flag = IterFlag::Converged;
                break;
            }
        }
    }
    let (x, flag) = melhor.escolhe(a, b, x, flag);
    Ok(finish(a, b, x, flag, it, resvec, nb))
}

/// MINRES (Paige & Saunders) para A simétrica, possivelmente indefinida; o precondicionador
/// deve ser simétrico definido positivo (Jacobi com diagonal positiva, IC(0)).
pub fn minres(a: &SparseMatrix, b: &[f64], opts: &IterOptions) -> Result<IterResult, SparseError> {
    let Setup {
        n,
        mut x,
        nb,
        max_iter,
        m,
    } = setup(a, b, opts)?;
    let mut r1 = vec![0.0; n];
    let nr0 = true_residual(a, b, &x, &mut r1);
    let mut resvec = vec![nr0];
    if nb == 0.0 {
        return Ok(finish(a, b, vec![0.0; n], IterFlag::Converged, 0, resvec, nb));
    }
    let alvo = opts.tol * nb;
    if nr0 <= alvo {
        return Ok(finish(a, b, x, IterFlag::Converged, 0, resvec, nb));
    }
    let mut y = vec![0.0; n];
    m.apply(&r1, &mut y);
    let ry = dot(&r1, &y);
    if ry <= 0.0 {
        return Ok(finish(a, b, x, IterFlag::Breakdown, 0, resvec, nb));
    }
    let beta1 = ry.sqrt();
    let (mut oldb, mut beta, mut dbar, mut epsln, mut phibar) = (0.0, beta1, 0.0, 0.0, beta1);
    let (mut cs, mut sn) = (-1.0f64, 0.0f64);
    let (mut w, mut w2) = (vec![0.0; n], vec![0.0; n]);
    let mut r2 = r1.clone();
    let mut v = vec![0.0; n];
    let mut flag = IterFlag::MaxIterations;
    let mut it = 0;
    let mut tmp = vec![0.0; n];
    while it < max_iter {
        it += 1;
        let s = 1.0 / beta;
        for i in 0..n {
            v[i] = s * y[i];
        }
        a.mul_vec_into(&v, &mut y);
        if it >= 2 {
            for i in 0..n {
                y[i] -= (beta / oldb) * r1[i];
            }
        }
        let alfa = dot(&v, &y);
        for i in 0..n {
            y[i] -= (alfa / beta) * r2[i];
        }
        r1.copy_from_slice(&r2);
        r2.copy_from_slice(&y);
        m.apply(&r2, &mut y);
        oldb = beta;
        let r2y = dot(&r2, &y);
        if r2y < 0.0 {
            flag = IterFlag::Breakdown;
            break;
        }
        beta = r2y.sqrt();
        let oldeps = epsln;
        let delta = cs * dbar + sn * alfa;
        let gbar = sn * dbar - cs * alfa;
        epsln = sn * beta;
        dbar = -cs * beta;
        let gamma = gbar.hypot(beta).max(f64::MIN_POSITIVE);
        cs = gbar / gamma;
        sn = beta / gamma;
        let phi = cs * phibar;
        phibar *= sn;
        // w = (v − ε·w₁ − δ·w₂)/γ, com w₁ ← w₂ ← w.
        let w1 = std::mem::replace(&mut w2, std::mem::take(&mut w));
        w = (0..n)
            .map(|i| (v[i] - oldeps * w1[i] - delta * w2[i]) / gamma)
            .collect();
        for i in 0..n {
            x[i] += phi * w[i];
        }
        resvec.push(phibar);
        // phibar estima o resíduo na norma de M⁻¹: confirma com o verdadeiro.
        if phibar <= alvo * (beta1 / nr0.max(f64::MIN_POSITIVE)) || beta == 0.0 {
            let nr = true_residual(a, b, &x, &mut tmp);
            if nr <= alvo {
                flag = IterFlag::Converged;
                break;
            }
            if beta == 0.0 {
                flag = IterFlag::Breakdown;
                break;
            }
        }
    }
    Ok(finish(a, b, x, flag, it, resvec, nb))
}

/// LSQR (C. C. Paige & M. A. Saunders, ACM TOMS 8 (1982) 43–71): mínimos quadrados
/// min ‖A·x − b‖² + λ²‖x‖² para A m×n esparsa (qualquer forma), por bidiagonalização de
/// Golub–Kahan. Sem precondicionador neste incremento.
///
/// Parada: ‖Aᵀr − λ²x‖ ≤ tol·‖A‖·‖r‖ (x é solução de mínimos quadrados) ou ‖r‖ ≤ tol·‖b‖
/// (sistema compatível). Com b = 0 devolve x = 0 sem iterar. O `relres` informado é ‖b − A·x‖/‖b‖.
///
/// Erros: `formaIncompativel` (b com tamanho ≠ m, x0 com tamanho ≠ n), `naoFinito`,
/// `opcaoInvalida` (tol, λ).
pub fn lsqr(a: &SparseMatrix, b: &[f64], damp: f64, opts: &IterOptions) -> Result<IterResult, SparseError> {
    let (m, n) = a.shape();
    if b.len() != m {
        return Err(SparseError::ShapeMismatch {
            expected: (m, 1),
            actual: (b.len(), 1),
        });
    }
    if !(opts.tol.is_finite() && opts.tol > 0.0) || !(damp.is_finite() && damp >= 0.0) {
        return Err(SparseError::InvalidOption("lsqr: tol > 0 e λ ≥ 0, finitos"));
    }
    if a.values().iter().chain(b).any(|v| !v.is_finite()) {
        return Err(SparseError::NonFinite);
    }
    let mut x = match &opts.x0 {
        Some(v) if v.len() != n => {
            return Err(SparseError::ShapeMismatch {
                expected: (n, 1),
                actual: (v.len(), 1),
            });
        }
        Some(v) => v.clone(),
        None => vec![0.0; n],
    };
    let nb = norm(b);
    if nb == 0.0 {
        // b = 0: x = 0 é a solução de mínima norma (com ou sem amortecimento), como nos outros
        // métodos. Regressão: com x₀ ≠ 0, ‖r‖ ≤ tol·‖b‖ = 0 nunca era alcançado.
        return Ok(finish_ls(
            a,
            b,
            vec![0.0; n],
            IterFlag::Converged,
            0,
            vec![0.0],
            nb,
        ));
    }
    let max_iter = opts.max_iter.unwrap_or(2 * n.max(m).max(50));
    // r₀ = b − A·x₀.
    let mut ax = vec![0.0; m];
    a.mul_vec_into(&x, &mut ax);
    let mut u: Vec<f64> = b.iter().zip(&ax).map(|(p, q)| p - q).collect();
    let mut beta = norm(&u);
    let mut resvec = vec![beta];
    if beta == 0.0 {
        return Ok(finish_ls(a, b, x, IterFlag::Converged, 0, resvec, nb));
    }
    u.iter_mut().for_each(|v| *v /= beta);
    let mut v = a.mul_vec_t(&u).expect("forma");
    let mut alpha = norm(&v);
    if alpha == 0.0 {
        // b ⟂ imagem de A: x₀ já minimiza.
        return Ok(finish_ls(a, b, x, IterFlag::Converged, 0, resvec, nb));
    }
    v.iter_mut().for_each(|t| *t /= alpha);
    let mut w = v.clone();
    let (mut phibar, mut rhobar) = (beta, alpha);
    let mut anorm2 = 0.0f64;
    let mut flag = IterFlag::MaxIterations;
    let mut it = 0;
    while it < max_iter {
        it += 1;
        // Bidiagonalização: β u = A v − α u; α v = Aᵀ u − β v.
        a.mul_vec_into(&v, &mut ax);
        for (ui, ai) in u.iter_mut().zip(&ax) {
            *ui = ai - alpha * *ui;
        }
        beta = norm(&u);
        if beta > 0.0 {
            u.iter_mut().for_each(|t| *t /= beta);
        }
        anorm2 += alpha * alpha + beta * beta + damp * damp;
        let atu = a.mul_vec_t(&u).expect("forma");
        for (vi, ai) in v.iter_mut().zip(&atu) {
            *vi = ai - beta * *vi;
        }
        alpha = norm(&v);
        if alpha > 0.0 {
            v.iter_mut().for_each(|t| *t /= alpha);
        }
        // Rotação para eliminar o amortecimento, depois a rotação plana do LSQR.
        let rhobar1 = rhobar.hypot(damp);
        let cs1 = rhobar / rhobar1;
        let phibar1 = cs1 * phibar;
        let rho = rhobar1.hypot(beta);
        let (cs, sn) = (rhobar1 / rho, beta / rho);
        let theta = sn * alpha;
        rhobar = -cs * alpha;
        let phi = cs * phibar1;
        phibar = sn * phibar1;
        for i in 0..n {
            x[i] += (phi / rho) * w[i];
            w[i] = v[i] - (theta / rho) * w[i];
        }
        // Estimativas: ‖r‖ ≈ φ̄ (com amortecimento, inclui λ‖x‖) e ‖Aᵀr‖ ≈ φ̄·α·|c|.
        // φ̄ pode ser negativo (a rotação do amortecimento preserva o sinal de ρ̄): a norma é
        // |φ̄| (regressão: comparar φ̄ com a tolerância declarava convergência cedo demais).
        let rnorm = phibar.abs();
        let arnorm = rnorm * alpha * cs.abs();
        resvec.push(rnorm);
        if rnorm <= opts.tol * nb || arnorm <= opts.tol * anorm2.sqrt() * rnorm || alpha == 0.0 {
            flag = IterFlag::Converged;
            break;
        }
    }
    Ok(finish_ls(a, b, x, flag, it, resvec, nb))
}

fn finish_ls(
    a: &SparseMatrix,
    b: &[f64],
    x: Vec<f64>,
    flag: IterFlag,
    iterations: usize,
    resvec: Vec<f64>,
    nb: f64,
) -> IterResult {
    let mut r = vec![0.0; b.len()];
    let nr = true_residual(a, b, &x, &mut r);
    let relres = if nb == 0.0 { nr } else { nr / nb };
    let flag = if x.iter().any(|v| !v.is_finite()) {
        IterFlag::NonFinite
    } else {
        flag
    };
    IterResult {
        x,
        flag,
        relres,
        iterations,
        resvec,
    }
}
