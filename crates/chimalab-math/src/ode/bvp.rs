//! Problemas de valor de contorno de dois pontos: y' = f(x, y), g(y(a), y(b)) = 0 (F050).
//!
//! Colocação de Lobatto IIIA de 3 estágios (Hermite–Simpson, ordem 4), como em J. Kierzenka &
//! L. F. Shampine, ACM TOMS 27 (2001) 299–316:
//! - em cada intervalo, y_{i+1} − y_i − h/6·(f_i + 4·f_m + f_{i+1}) = 0, com
//!   y_m = (y_i + y_{i+1})/2 − h/8·(f_{i+1} − f_i);
//! - o sistema não linear (equações dos intervalos mais as de contorno) é resolvido por Newton
//!   com Jacobiano em blocos por diferenças finitas, montado esparso e fatorado pela LU esparsa
//!   (F041);
//! - a malha é refinada onde o resíduo da solução contínua, S'(x) − f(x, S(x)), excede a
//!   tolerância (S é a Hermite cúbica por y_i e f_i), medido em pontos de Gauss de cada intervalo
//!   e escalado pelo tamanho de cada componente de f na malha inteira.

use super::OdeError;
use crate::sparse::{SparseMatrix, spsolve};

/// Opções.
#[derive(Clone, Debug, PartialEq)]
pub struct BvpOptions {
    /// Tolerância relativa do resíduo (padrão 1e-6).
    pub rel_tol: f64,
    /// Tolerância absoluta do resíduo (padrão 1e-9).
    pub abs_tol: f64,
    /// Máximo de pontos da malha (padrão 5000).
    pub max_mesh: usize,
    /// Máximo de iterações de Newton por malha (padrão 50).
    pub max_newton: usize,
}

impl Default for BvpOptions {
    fn default() -> Self {
        Self {
            rel_tol: 1e-6,
            abs_tol: 1e-9,
            max_mesh: 5000,
            max_newton: 50,
        }
    }
}

/// Resultado.
#[derive(Clone, Debug, PartialEq)]
pub struct BvpSolution {
    /// Malha final.
    pub x: Vec<f64>,
    /// Solução nos nós.
    pub y: Vec<Vec<f64>>,
    /// Derivadas f(x, y) nos nós (para a interpolação cúbica).
    pub yp: Vec<Vec<f64>>,
    /// Maior resíduo relativo estimado.
    pub max_residual: f64,
    /// Convergiu dentro da tolerância.
    pub converged: bool,
    /// Iterações de Newton (total).
    pub newton_iterations: usize,
}

impl BvpSolution {
    /// Solução contínua (Hermite cúbica) em x ∈ [a, b]; `None` fora.
    pub fn eval(&self, x: f64) -> Option<Vec<f64>> {
        let (a, b) = (self.x[0], *self.x.last()?);
        if !(x >= a && x <= b) {
            return None;
        }
        let i = self.x.partition_point(|&v| v <= x).clamp(1, self.x.len() - 1) - 1;
        Some(
            hermite(
                self.x[i],
                self.x[i + 1],
                &self.y[i],
                &self.y[i + 1],
                &self.yp[i],
                &self.yp[i + 1],
                x,
            )
            .0,
        )
    }
}

/// Hermite cúbica e sua derivada em x.
fn hermite(x0: f64, x1: f64, y0: &[f64], y1: &[f64], f0: &[f64], f1: &[f64], x: f64) -> (Vec<f64>, Vec<f64>) {
    let h = x1 - x0;
    let t = (x - x0) / h;
    let (h00, h10, h01, h11) = (
        2.0 * t * t * t - 3.0 * t * t + 1.0,
        t * t * t - 2.0 * t * t + t,
        -2.0 * t * t * t + 3.0 * t * t,
        t * t * t - t * t,
    );
    let (d00, d10, d01, d11) = (
        6.0 * t * t - 6.0 * t,
        3.0 * t * t - 4.0 * t + 1.0,
        -6.0 * t * t + 6.0 * t,
        3.0 * t * t - 2.0 * t,
    );
    let n = y0.len();
    let v = (0..n)
        .map(|c| h00 * y0[c] + h * h10 * f0[c] + h01 * y1[c] + h * h11 * f1[c])
        .collect();
    let d = (0..n)
        .map(|c| (d00 * y0[c] + h * d10 * f0[c] + d01 * y1[c] + h * d11 * f1[c]) / h)
        .collect();
    (v, d)
}

/// Resolve o problema de contorno a partir da malha e do chute iniciais.
///
/// - `f(x, y, dy)`: lado direito (n componentes).
/// - `bc(ya, yb, res)`: n condições de contorno, com res = 0 na solução.
/// - `mesh`: malha inicial estritamente crescente (≥ 2 pontos); `guess(x, y)`: chute inicial.
///
/// Erros: `intervaloInvalido` (malha), `condicaoInicialInvalida` (n = 0, chute não finito),
/// `toleranciaInvalida`, `opcaoInvalida`. A não convergência não é erro: o resultado vem com
/// `converged = false`.
pub fn bvp(
    mut f: impl FnMut(f64, &[f64], &mut [f64]),
    mut bc: impl FnMut(&[f64], &[f64], &mut [f64]),
    mesh: &[f64],
    guess: impl Fn(f64, &mut [f64]),
    n: usize,
    opts: &BvpOptions,
) -> Result<BvpSolution, OdeError> {
    if mesh.len() < 2 || mesh.iter().any(|v| !v.is_finite()) || mesh.windows(2).any(|w| w[1] <= w[0]) {
        return Err(OdeError::InvalidTspan(
            "bvp: malha finita, crescente, com ≥ 2 pontos",
        ));
    }
    if n == 0 {
        return Err(OdeError::InvalidStart("estado vazio"));
    }
    if !(opts.rel_tol.is_finite() && opts.rel_tol > 0.0 && opts.abs_tol.is_finite() && opts.abs_tol >= 0.0) {
        return Err(OdeError::InvalidTolerance(
            "bvp: rel_tol > 0 e abs_tol ≥ 0, finitas",
        ));
    }
    if opts.max_mesh < mesh.len() || opts.max_newton == 0 {
        return Err(OdeError::InvalidOption(
            "bvp: max_mesh ≥ pontos da malha e max_newton ≥ 1",
        ));
    }
    let mut x: Vec<f64> = mesh.to_vec();
    let mut y: Vec<Vec<f64>> = x
        .iter()
        .map(|&xi| {
            let mut v = vec![0.0; n];
            guess(xi, &mut v);
            v
        })
        .collect();
    if y.iter().flatten().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("chute inicial não finito"));
    }
    let mut total_newton = 0;
    loop {
        // Newton na malha atual.
        let mut ok = false;
        for _ in 0..opts.max_newton {
            total_newton += 1;
            let (res, jac) = sistema(&mut f, &mut bc, &x, &y, n);
            let nr = res.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            if !nr.is_finite() {
                break;
            }
            let Ok((dy, _)) = spsolve(&jac, &res.iter().map(|v| -v).collect::<Vec<_>>()) else {
                break;
            };
            let passo = dy.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            let escala = y.iter().flatten().fold(1.0f64, |m, v| m.max(v.abs()));
            // Newton amortecido simples: reduz o passo se o resíduo não cair.
            let mut lambda = 1.0;
            let mut melhor = None;
            for _ in 0..12 {
                let yt: Vec<Vec<f64>> = (0..x.len())
                    .map(|i| (0..n).map(|c| y[i][c] + lambda * dy[i * n + c]).collect())
                    .collect();
                let (rt, _) = residuos(&mut f, &mut bc, &x, &yt, n);
                let nt = rt.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                if nt.is_finite() && nt < nr.max(f64::MIN_POSITIVE) * (1.0 - 1e-4 * lambda) || nr == 0.0 {
                    melhor = Some(yt);
                    break;
                }
                lambda /= 2.0;
            }
            let Some(yt) = melhor else {
                // Nenhum passo reduz o resíduo, mas o passo completo já é desprezível: piso de
                // arredondamento, convergiu. Regressão: Mathieu com parâmetro desconhecido tinha λ
                // certo e resíduo 1e-9, mas o Newton nunca era dado como convergido, e a malha
                // inteira era refinada até 2893 pontos.
                ok = passo <= 1e-8 * escala;
                break;
            };
            y = yt;
            if passo * lambda <= 1e-12 * escala || nr <= 1e-14 * escala {
                ok = true;
                break;
            }
        }
        // Resíduo da solução contínua em 2 pontos de Gauss por intervalo.
        let fs: Vec<Vec<f64>> = x
            .iter()
            .zip(&y)
            .map(|(&xi, yi)| {
                let mut d = vec![0.0; n];
                f(xi, yi, &mut d);
                d
            })
            .collect();
        let g = 0.5 / 3f64.sqrt();
        // Escala por componente: o tamanho de f na malha inteira. Uma escala pontual exigiria
        // precisão absurda onde f cruza zero e refinaria à toa.
        let escala: Vec<f64> = (0..n)
            .map(|c| opts.abs_tol + opts.rel_tol * fs.iter().fold(0.0f64, |m, v| m.max(v[c].abs())))
            .collect();
        let mut piores = vec![0.0f64; x.len() - 1];
        for i in 0..x.len() - 1 {
            let h = x[i + 1] - x[i];
            for t in [0.5 - g, 0.5 + g] {
                let xs = x[i] + t * h;
                let (s, sp) = hermite(x[i], x[i + 1], &y[i], &y[i + 1], &fs[i], &fs[i + 1], xs);
                let mut fv = vec![0.0; n];
                f(xs, &s, &mut fv);
                for c in 0..n {
                    // Resíduo relativo: |S' − f| na escala de f, em unidades de rel_tol.
                    piores[i] = piores[i].max((sp[c] - fv[c]).abs() / escala[c] * opts.rel_tol);
                }
            }
        }
        let maxr = piores.iter().copied().fold(0.0f64, f64::max);
        let dentro = maxr <= opts.rel_tol;
        if ok && dentro {
            return Ok(BvpSolution {
                x,
                y,
                yp: fs,
                max_residual: maxr,
                converged: true,
                newton_iterations: total_newton,
            });
        }
        // Refina: divide em 2 (ou 3, se o resíduo for 100× a tolerância) os intervalos ruins.
        let ruins: Vec<usize> = (0..piores.len())
            .filter(|&i| !ok || piores[i] > opts.rel_tol)
            .collect();
        let novos: usize = ruins
            .iter()
            .map(|&i| if piores[i] > 100.0 * opts.rel_tol { 2 } else { 1 })
            .sum();
        if novos == 0 || x.len() + novos > opts.max_mesh {
            return Ok(BvpSolution {
                x,
                y,
                yp: fs,
                max_residual: maxr,
                converged: false,
                newton_iterations: total_newton,
            });
        }
        let mut nx = Vec::with_capacity(x.len() + novos);
        let mut ny = Vec::with_capacity(x.len() + novos);
        for i in 0..x.len() - 1 {
            nx.push(x[i]);
            ny.push(y[i].clone());
            let partes = if ruins.binary_search(&i).is_ok() {
                if piores[i] > 100.0 * opts.rel_tol { 3 } else { 2 }
            } else {
                1
            };
            for p in 1..partes {
                let xs = x[i] + (x[i + 1] - x[i]) * p as f64 / partes as f64;
                nx.push(xs);
                ny.push(hermite(x[i], x[i + 1], &y[i], &y[i + 1], &fs[i], &fs[i + 1], xs).0);
            }
        }
        nx.push(*x.last().expect("não vazio"));
        ny.push(y.last().expect("não vazio").clone());
        x = nx;
        y = ny;
    }
}

/// Resíduos das equações de colocação e de contorno.
fn residuos(
    f: &mut impl FnMut(f64, &[f64], &mut [f64]),
    bc: &mut impl FnMut(&[f64], &[f64], &mut [f64]),
    x: &[f64],
    y: &[Vec<f64>],
    n: usize,
) -> (Vec<f64>, Vec<Vec<f64>>) {
    let m = x.len();
    let fs: Vec<Vec<f64>> = x
        .iter()
        .zip(y)
        .map(|(&xi, yi)| {
            let mut d = vec![0.0; n];
            f(xi, yi, &mut d);
            d
        })
        .collect();
    let mut r = Vec::with_capacity(n * m);
    for i in 0..m - 1 {
        r.extend(intervalo(f, x[i], x[i + 1], &y[i], &y[i + 1], &fs[i], &fs[i + 1]));
    }
    let mut g = vec![0.0; n];
    bc(&y[0], &y[m - 1], &mut g);
    r.extend(g);
    (r, fs)
}

/// Resíduo de Hermite–Simpson de um intervalo.
fn intervalo(
    f: &mut impl FnMut(f64, &[f64], &mut [f64]),
    x0: f64,
    x1: f64,
    y0: &[f64],
    y1: &[f64],
    f0: &[f64],
    f1: &[f64],
) -> Vec<f64> {
    let n = y0.len();
    let h = x1 - x0;
    let ym: Vec<f64> = (0..n)
        .map(|c| 0.5 * (y0[c] + y1[c]) - h / 8.0 * (f1[c] - f0[c]))
        .collect();
    let mut fm = vec![0.0; n];
    f(x0 + 0.5 * h, &ym, &mut fm);
    (0..n)
        .map(|c| y1[c] - y0[c] - h / 6.0 * (f0[c] + 4.0 * fm[c] + f1[c]))
        .collect()
}

/// Resíduos e Jacobiano esparso (blocos por diferenças finitas).
fn sistema(
    f: &mut impl FnMut(f64, &[f64], &mut [f64]),
    bc: &mut impl FnMut(&[f64], &[f64], &mut [f64]),
    x: &[f64],
    y: &[Vec<f64>],
    n: usize,
) -> (Vec<f64>, SparseMatrix) {
    let m = x.len();
    let (r, fs) = residuos(f, bc, x, y, n);
    let mut t: Vec<(usize, usize, f64)> = Vec::new();
    let delta = |v: f64| (f64::EPSILON * v.abs().max(1.0)).sqrt();
    let deriv = |f: &mut super::Rhs<'_>, xi: f64, yi: &[f64]| {
        let mut d = vec![0.0; n];
        f(xi, yi, &mut d);
        d
    };
    for i in 0..m - 1 {
        let base = &r[i * n..(i + 1) * n];
        for (lado, col0) in [(0usize, i * n), (1usize, (i + 1) * n)] {
            for c in 0..n {
                let mut y0 = y[i].clone();
                let mut y1 = y[i + 1].clone();
                let alvo = if lado == 0 { &mut y0 } else { &mut y1 };
                let dlt = delta(alvo[c]);
                alvo[c] += dlt;
                let f0 = if lado == 0 {
                    deriv(f, x[i], &y0)
                } else {
                    fs[i].clone()
                };
                let f1 = if lado == 1 {
                    deriv(f, x[i + 1], &y1)
                } else {
                    fs[i + 1].clone()
                };
                let ri = intervalo(f, x[i], x[i + 1], &y0, &y1, &f0, &f1);
                for (lin, (a, b)) in ri.iter().zip(base).enumerate() {
                    let v = (a - b) / dlt;
                    if v != 0.0 {
                        t.push((i * n + lin, col0 + c, v));
                    }
                }
            }
        }
    }
    // Contorno: depende de y(a) e y(b).
    let base = &r[(m - 1) * n..];
    for (no, col0) in [(0usize, 0usize), (m - 1, (m - 1) * n)] {
        for c in 0..n {
            let (mut ya, mut yb) = (y[0].clone(), y[m - 1].clone());
            let alvo = if no == 0 { &mut ya } else { &mut yb };
            let dlt = delta(alvo[c]);
            alvo[c] += dlt;
            let mut g = vec![0.0; n];
            bc(&ya, &yb, &mut g);
            for (lin, (a, b)) in g.iter().zip(base).enumerate() {
                let v = (a - b) / dlt;
                if v != 0.0 {
                    t.push(((m - 1) * n + lin, col0 + c, v));
                }
            }
        }
    }
    (
        r,
        SparseMatrix::from_triplets(n * m, n * m, &t).expect("índices válidos"),
    )
}

/// Problema de contorno com `np` parâmetros desconhecidos p (autovalores, períodos, constantes):
/// y' = f(x, y, p), g(y(a), y(b), p) = 0 com n + np condições.
///
/// Redução clássica: cada parâmetro vira uma componente a mais com p' = 0. A colocação mantém p
/// exatamente constante, porque o resíduo do intervalo é p_{i+1} − p_i. Devolve a solução (só as
/// n componentes de y) e os parâmetros.
///
/// Erros: os de [`bvp`], e `condicaoInicialInvalida` se `p0` não for finito.
#[allow(clippy::too_many_arguments)]
pub fn bvp_params(
    mut f: impl FnMut(f64, &[f64], &[f64], &mut [f64]),
    mut bc: impl FnMut(&[f64], &[f64], &[f64], &mut [f64]),
    mesh: &[f64],
    guess: impl Fn(f64, &mut [f64]),
    n: usize,
    p0: &[f64],
    opts: &BvpOptions,
) -> Result<(BvpSolution, Vec<f64>), OdeError> {
    if p0.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("parâmetros iniciais não finitos"));
    }
    if n == 0 {
        return Err(OdeError::InvalidStart("estado vazio"));
    }
    let np = p0.len();
    let mut sol = bvp(
        |x, z, dz| {
            let (y, p) = z.split_at(n);
            f(x, y, p, &mut dz[..n]);
            dz[n..].fill(0.0);
        },
        |za, zb, r| bc(&za[..n], &zb[..n], &za[n..], r),
        mesh,
        |x, z| {
            guess(x, &mut z[..n]);
            z[n..].copy_from_slice(p0);
        },
        n + np,
        opts,
    )?;
    let p = sol.y[0][n..].to_vec();
    for v in sol.y.iter_mut().chain(sol.yp.iter_mut()) {
        v.truncate(n);
    }
    Ok((sol, p))
}
