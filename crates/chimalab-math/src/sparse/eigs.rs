//! Autovalores parciais de matrizes esparsas simétricas: Krylov–Schur simétrico, isto é, Lanczos
//! com reinício espesso (K. Wu & H. Simon, SIAM J. Matrix Anal. Appl. 22 (2000) 602–616;
//! G. W. Stewart, SIAM J. Matrix Anal. Appl. 23 (2001) 601–614), com reortogonalização completa
//! e deslocamento-inversão para autovalores perto de σ.
//!
//! Autovalores múltiplos: com um único vetor inicial, o espaço de Krylov só contém uma direção
//! de cada autoespaço. Depois de achar k pares, o método procura de novo no complemento
//! ortogonal deles (deflação com travamento); um autovalor melhor que o pior achado (a cópia que
//! faltava) entra no lugar dele, e a verificação se repete.

use faer::linalg::solvers::Solve;
use faer::sparse::{SparseColMat, Triplet};
use faer::{Mat, MatMut, Side};

use super::{SparseError, SparseMatrix};
use crate::random::{Algorithm, Rng};

/// Quais autovalores.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Which {
    /// Maiores em módulo (padrão).
    #[default]
    LargestMagnitude,
    /// Maiores (algébricos).
    LargestAlgebraic,
    /// Menores (algébricos).
    SmallestAlgebraic,
    /// Mais próximos de σ (deslocamento-inversão: fatora A − σI uma vez).
    NearestTo(f64),
}

/// Opções.
#[derive(Clone, Debug, PartialEq)]
pub struct EigsOptions {
    /// Tolerância relativa do resíduo ‖A·x − λ·x‖ ≤ tol·‖A‖ (padrão 1e-10).
    pub tol: f64,
    /// Dimensão do subespaço (padrão min(n, max(2k + 1, 20))).
    pub subspace: Option<usize>,
    /// Máximo de reinícios (padrão 300).
    pub max_restarts: usize,
    /// Semente do vetor inicial (determinístico).
    pub seed: u64,
}

impl Default for EigsOptions {
    fn default() -> Self {
        Self {
            tol: 1e-10,
            subspace: None,
            max_restarts: 300,
            seed: 2026,
        }
    }
}

/// Resultado.
#[derive(Clone, Debug, PartialEq)]
pub struct EigsResult {
    /// Autovalores, na ordem de [`Which`] (o primeiro é o "mais" pedido).
    pub values: Vec<f64>,
    /// Autovetores unitários (um por autovalor).
    pub vectors: Vec<Vec<f64>>,
    /// Resíduos ‖A·x − λ·x‖ verdadeiros.
    pub residuals: Vec<f64>,
    /// Todos os k convergiram.
    pub converged: bool,
    /// Reinícios usados.
    pub restarts: usize,
    /// Produtos A·v (ou soluções com A − σI).
    pub operations: usize,
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// k autovalores de A simétrica.
///
/// Erros: `naoQuadrada`, `opcaoInvalida` (A não simétrica, k = 0 ou k ≥ n, tol inválida),
/// `naoFinito`, `singular` (A − σI singular no deslocamento-inversão).
pub fn eigs(a: &SparseMatrix, k: usize, which: Which, opts: &EigsOptions) -> Result<EigsResult, SparseError> {
    let (n, c) = a.shape();
    if n != c {
        return Err(SparseError::NotSquare((n, c)));
    }
    if !a.is_symmetric() {
        return Err(SparseError::InvalidOption(
            "eigs: matriz simétrica (o caso geral é incremento futuro)",
        ));
    }
    if k == 0 || k >= n {
        return Err(SparseError::InvalidOption("eigs: 1 ≤ k < n"));
    }
    if !(opts.tol.is_finite() && opts.tol > 0.0) {
        return Err(SparseError::InvalidOption("eigs: tol finita e > 0"));
    }
    if a.values().iter().any(|v| !v.is_finite()) {
        return Err(SparseError::NonFinite);
    }
    // Operador: A, ou (A − σI)⁻¹ pela LU esparsa fatorada uma vez.
    let lu = match which {
        Which::NearestTo(s) => {
            if !s.is_finite() {
                return Err(SparseError::InvalidOption("eigs: σ finito"));
            }
            let mut t: Vec<Triplet<usize, usize, f64>> = a
                .triplets()
                .into_iter()
                .map(|(i, j, v)| Triplet::new(i, j, v))
                .collect();
            t.extend((0..n).map(|i| Triplet::new(i, i, -s)));
            let m = SparseColMat::try_new_from_triplets(n, n, &t).expect("triplas válidas");
            Some(m.sp_lu().map_err(|_| SparseError::Singular)?)
        }
        _ => None,
    };
    let ops = std::cell::Cell::new(0usize);
    let apply = |v: &[f64], w: &mut [f64]| -> bool {
        ops.set(ops.get() + 1);
        match &lu {
            Some(f) => {
                w.copy_from_slice(v);
                f.solve_in_place(MatMut::from_column_major_slice_mut(w, n, 1));
                w.iter().all(|x| x.is_finite())
            }
            None => {
                a.mul_vec_into(v, w);
                true
            }
        }
    };
    // Ordem de preferência dos valores de Ritz do operador (menor = melhor).
    let rank = |t: f64| -> f64 {
        match which {
            Which::LargestMagnitude | Which::NearestTo(_) => -t.abs(),
            Which::LargestAlgebraic => -t,
            Which::SmallestAlgebraic => t,
        }
    };
    let norm_a = a.values().iter().fold(0.0f64, |s, v| s.max(v.abs())) * (n as f64).sqrt();
    let mut rng = Rng::new(Algorithm::Philox, opts.seed);
    let mut restarts = 0usize;
    // Primeira busca: k pares; depois, verificações de 1 par no complemento dos achados.
    let mut achados: Vec<(f64, Vec<f64>, bool)> = ks_run(
        &apply,
        n,
        k,
        &[],
        opts,
        &rank,
        norm_a,
        which,
        &mut rng,
        &mut restarts,
    )?;
    for _ in 0..k {
        if achados.len() >= n {
            break;
        }
        let travados: Vec<Vec<f64>> = achados.iter().map(|p| p.1.clone()).collect();
        let extra = ks_run(
            &apply,
            n,
            1,
            &travados,
            opts,
            &rank,
            norm_a,
            which,
            &mut rng,
            &mut restarts,
        )?;
        let Some(e) = extra.into_iter().next() else { break };
        let pior = achados
            .iter()
            .map(|p| rank(p.0))
            .fold(f64::NEG_INFINITY, f64::max);
        if e.2 && rank(e.0) < pior - 1e-12 * e.0.abs().max(1e-300) {
            let ip = achados.iter().position(|p| rank(p.0) == pior).expect("existe");
            achados[ip] = e;
        } else {
            break;
        }
    }
    // Autovalores de A pelo quociente de Rayleigh, na ordem pedida.
    let mut pares: Vec<(f64, Vec<f64>)> = achados
        .into_iter()
        .map(|(_, x, _)| {
            let mut ax = vec![0.0; n];
            a.mul_vec_into(&x, &mut ax);
            (dot(&x, &ax), x)
        })
        .collect();
    let chave = |l: f64| match which {
        Which::LargestMagnitude => -l.abs(),
        Which::LargestAlgebraic => -l,
        Which::SmallestAlgebraic => l,
        Which::NearestTo(s) => (l - s).abs(),
    };
    pares.sort_by(|p, q| chave(p.0).total_cmp(&chave(q.0)));
    let residuals: Vec<f64> = pares
        .iter()
        .map(|(l, x)| {
            let mut ax = vec![0.0; n];
            a.mul_vec_into(x, &mut ax);
            ax.iter()
                .zip(x)
                .map(|(p, q)| (p - l * q).powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .collect();
    let converged = pares.len() == k
        && residuals
            .iter()
            .all(|r| *r <= 1e3 * opts.tol * norm_a.max(1e-300));
    Ok(EigsResult {
        values: pares.iter().map(|p| p.0).collect(),
        vectors: pares.into_iter().map(|p| p.1).collect(),
        residuals,
        converged,
        restarts,
        operations: ops.get(),
    })
}

/// Vetor aleatório unitário ortogonal a `base` (Gram–Schmidt duas vezes).
fn aleatorio_ortogonal(n: usize, base: &[Vec<f64>], rng: &mut Rng) -> Option<Vec<f64>> {
    for _ in 0..10 {
        let mut v: Vec<f64> = (0..n).map(|_| rng.uniform() - 0.5).collect();
        for _ in 0..2 {
            for b in base {
                let c = dot(b, &v);
                v.iter_mut().zip(b).for_each(|(x, y)| *x -= c * y);
            }
        }
        let nv = dot(&v, &v).sqrt();
        if nv > 1e-8 {
            v.iter_mut().for_each(|x| *x /= nv);
            return Some(v);
        }
    }
    None
}

/// Uma execução de Krylov–Schur simétrico no complemento ortogonal de `travados`: devolve até
/// `k` pares (valor de Ritz do operador, vetor, convergiu).
#[allow(clippy::too_many_arguments)]
fn ks_run(
    apply: &dyn Fn(&[f64], &mut [f64]) -> bool,
    n: usize,
    k: usize,
    travados: &[Vec<f64>],
    opts: &EigsOptions,
    rank: &dyn Fn(f64) -> f64,
    norm_a: f64,
    which: Which,
    rng: &mut Rng,
    restarts: &mut usize,
) -> Result<Vec<(f64, Vec<f64>, bool)>, SparseError> {
    let livre = n - travados.len();
    if livre == 0 {
        return Ok(Vec::new());
    }
    let k = k.min(livre);
    let m = opts.subspace.unwrap_or((2 * k + 1).max(20)).min(livre).max(k);
    let Some(v0) = aleatorio_ortogonal(n, travados, rng) else {
        return Ok(Vec::new());
    };
    let mut vs: Vec<Vec<f64>> = vec![v0];
    let mut h = vec![vec![0.0; m]; m + 1];
    let mut j0 = 0usize;
    let mut w = vec![0.0; n];
    let mut locais = 0usize;
    loop {
        let mut mm = m;
        for j in j0..m {
            if !apply(&vs[j], &mut w) {
                return Err(SparseError::Singular);
            }
            // Gram–Schmidt completo (base e travados), duas passadas.
            for _ in 0..2 {
                for t in travados {
                    let c = dot(t, &w);
                    w.iter_mut().zip(t).for_each(|(x, y)| *x -= c * y);
                }
                for (i, vi) in vs.iter().enumerate() {
                    let c = dot(vi, &w);
                    h[i][j] += c;
                    w.iter_mut().zip(vi).for_each(|(x, y)| *x -= c * y);
                }
            }
            let beta = dot(&w, &w).sqrt();
            if vs.len() == livre {
                mm = j + 1;
                h[j + 1][j] = 0.0;
                break;
            }
            if beta <= 1e-14 * norm_a.max(f64::MIN_POSITIVE) {
                // Subespaço invariante: continua com um vetor novo ortogonal a tudo (acoplamento 0).
                let mut base = travados.to_vec();
                base.extend(vs.iter().cloned());
                match aleatorio_ortogonal(n, &base, rng) {
                    Some(v) => {
                        h[j + 1][j] = 0.0;
                        vs.push(v);
                        continue;
                    }
                    None => {
                        mm = j + 1;
                        h[j + 1][j] = 0.0;
                        break;
                    }
                }
            }
            h[j + 1][j] = beta;
            vs.push(w.iter().map(|x| x / beta).collect());
        }
        // Rayleigh–Ritz na parte quadrada (simetrizada contra o arredondamento).
        let hm = Mat::from_fn(mm, mm, |i, j| 0.5 * (h[i][j] + h[j][i]));
        let eig = hm
            .self_adjoint_eigen(Side::Lower)
            .expect("autovalores de matriz simétrica pequena");
        let theta: Vec<f64> = (0..mm).map(|i| eig.S()[i]).collect();
        let mut ordem: Vec<usize> = (0..mm).collect();
        ordem.sort_by(|&p, &q| rank(theta[p]).total_cmp(&rank(theta[q])));
        let s_col = |i: usize| -> Vec<f64> { (0..mm).map(|r| eig.U()[(r, i)]).collect() };
        // Resíduo de cada par de Ritz: |linha mm de H · s| (0 se a base esgotou o espaço).
        let res = |i: usize| -> f64 {
            if mm < m || vs.len() <= mm {
                return 0.0;
            }
            let s = s_col(i);
            (0..mm).map(|c| h[mm][c] * s[c]).sum::<f64>().abs()
        };
        let escala = |t: f64| match which {
            Which::NearestTo(_) => t.abs(),
            _ => norm_a,
        };
        let conv = |i: usize| res(i) <= opts.tol * escala(theta[i]).max(f64::MIN_POSITIVE);
        let kk = k.min(mm);
        if ordem[..kk].iter().all(|&i| conv(i)) || locais >= opts.max_restarts || mm < m {
            return Ok(ordem[..kk]
                .iter()
                .map(|&i| {
                    let s = s_col(i);
                    let mut x = vec![0.0; n];
                    for (c, sc) in s.iter().enumerate() {
                        x.iter_mut().zip(&vs[c]).for_each(|(xl, vl)| *xl += sc * vl);
                    }
                    let nx = dot(&x, &x).sqrt();
                    x.iter_mut().for_each(|v| *v /= nx);
                    (theta[i], x, conv(i))
                })
                .collect());
        }
        // Reinício espesso: mantém p vetores de Ritz preferidos e o último vetor da base.
        *restarts += 1;
        locais += 1;
        let p = (k + (m - k) / 2).min(m - 1).max(1);
        let mut novos: Vec<Vec<f64>> = Vec::with_capacity(m + 1);
        let mut hn = vec![vec![0.0; m]; m + 1];
        for (r, &i) in ordem[..p].iter().enumerate() {
            let s = s_col(i);
            let mut y = vec![0.0; n];
            for (c, sc) in s.iter().enumerate() {
                y.iter_mut().zip(&vs[c]).for_each(|(yl, vl)| *yl += sc * vl);
            }
            novos.push(y);
            hn[r][r] = theta[i];
            hn[p][r] = (0..mm).map(|c| h[mm][c] * s[c]).sum();
        }
        novos.push(vs[mm].clone());
        vs = novos;
        h = hn;
        j0 = p;
    }
}

/// Resultado de [`svds`].
#[derive(Clone, Debug, PartialEq)]
pub struct SvdsResult {
    /// Valores singulares, decrescentes.
    pub values: Vec<f64>,
    /// Vetores singulares à esquerda (m elementos cada).
    pub left: Vec<Vec<f64>>,
    /// Vetores singulares à direita (n elementos cada).
    pub right: Vec<Vec<f64>>,
    /// Todos convergiram.
    pub converged: bool,
}

/// Os k maiores valores singulares de A (m×n), pelos autovalores ±σ da matriz aumentada
/// simétrica [[0, A], [Aᵀ, 0]]. Isso evita formar AᵀA, que elevaria o número de condição ao
/// quadrado.
///
/// Erros: `opcaoInvalida` (k = 0 ou k > min(m, n)), `naoFinito`.
pub fn svds(a: &SparseMatrix, k: usize, opts: &EigsOptions) -> Result<SvdsResult, SparseError> {
    let (m, n) = a.shape();
    if k == 0 || k > m.min(n) {
        return Err(SparseError::InvalidOption("svds: 1 ≤ k ≤ min(m, n)"));
    }
    let mut t: Vec<(usize, usize, f64)> = Vec::with_capacity(2 * a.nnz());
    for (i, j, v) in a.triplets() {
        t.push((i, m + j, v));
        t.push((m + j, i, v));
    }
    let aug = SparseMatrix::from_triplets(m + n, m + n, &t).expect("índices válidos");
    // Pede os 2k maiores em módulo (±σ) e fica com os positivos.
    let kk = (2 * k).min(m + n - 1);
    let r = eigs(&aug, kk, Which::LargestAlgebraic, opts)?;
    let mut pares: Vec<(f64, Vec<f64>)> = r
        .values
        .into_iter()
        .zip(r.vectors)
        .filter(|(l, _)| *l > 0.0)
        .collect();
    pares.sort_by(|p, q| q.0.total_cmp(&p.0));
    pares.truncate(k);
    let norm = |v: &[f64]| v.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mut left = Vec::with_capacity(k);
    let mut right = Vec::with_capacity(k);
    for (_, x) in &pares {
        let (u, v) = x.split_at(m);
        let (nu, nv) = (norm(u), norm(v));
        left.push(u.iter().map(|z| z / nu).collect());
        right.push(v.iter().map(|z| z / nv).collect());
    }
    Ok(SvdsResult {
        values: pares.iter().map(|p| p.0).collect(),
        left,
        right,
        converged: r.converged && pares.len() == k,
    })
}
