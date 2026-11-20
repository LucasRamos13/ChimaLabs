//! Fórmulas de diferenças regressivas (BDF) e numéricas (NDF) de ordem variável 1–5, para
//! problemas rígidos.
//!
//! Fontes:
//! - L. F. Shampine & M. W. Reichelt, SIAM J. Sci. Comput. 18 (1997) 1–22. Daí vêm as NDF (constantes κ da tabela 1), a forma de passo quase constante com diferenças regressivas, a troca de passo pelas matrizes R e U (§2.3), a iteração de Newton simplificada e a escolha de ordem.
//! - E. Hairer & G. Wanner, *Solving ODE II*, 2.ª ed. (1996), §IV.8, para o incremento do Jacobiano por diferenças.
//!
//! O estado é a matriz de diferenças D: D₀ = y, D₁ = h·y' e Dⱼ = ∇ʲ⁻¹(h·y'), na escala do passo atual.

use faer::linalg::solvers::{PartialPivLu, Solve};
use faer::{Mat, MatMut};

use super::{Interp, OdeStats, OdeStatus, Outcome, Rhs, Stepper, signed};
use crate::linalg::Matrix;

const MAX_ORDER: usize = 5;
const NEWTON_MAXITER: usize = 4;
const MIN_FACTOR: f64 = 0.2;
const MAX_FACTOR: f64 = 10.0;

/// Jacobiano ∂f/∂y fornecido pelo usuário: preenche a matriz n×n.
pub type JacFn<'a> = Box<dyn FnMut(f64, &[f64], &mut Matrix) + 'a>;

pub(crate) struct Bdf<'j, 'a> {
    n: usize,
    dir: f64,
    t: f64,
    t_bound: f64,
    h_abs: f64,
    hmax: f64,
    /// Diferenças D₀..D_{MAX_ORDER+2}.
    d: Vec<Vec<f64>>,
    order: usize,
    max_order: usize,
    n_equal_steps: usize,
    gamma: [f64; MAX_ORDER + 2],
    alpha: [f64; MAX_ORDER + 2],
    error_const: [f64; MAX_ORDER + 2],
    rtol: f64,
    atol: Vec<f64>,
    newton_tol: f64,
    jac_fn: Option<&'j mut JacFn<'a>>,
    jac: Mat<f64>,
    jac_current: bool,
    lu: Option<PartialPivLu<f64>>,
    nonfinite: bool,
}

impl<'j, 'a> Bdf<'j, 'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        f: &mut Rhs<'_>,
        t0: f64,
        y0: &[f64],
        f0: &[f64],
        t_bound: f64,
        h0: f64,
        hmax: f64,
        rtol: f64,
        atol: Vec<f64>,
        ndf: bool,
        max_order: usize,
        jac_fn: Option<&'j mut JacFn<'a>>,
        stats: &mut OdeStats,
    ) -> Self {
        let n = y0.len();
        let dir = if t_bound > t0 { 1.0 } else { -1.0 };
        // κ das NDF (Shampine & Reichelt, tabela 1); nulos dão as BDF.
        let kappa: [f64; MAX_ORDER + 2] = if ndf {
            [0.0, -0.1850, -1.0 / 9.0, -0.0823, -0.0415, 0.0, 0.0]
        } else {
            [0.0; MAX_ORDER + 2]
        };
        let mut gamma = [0.0; MAX_ORDER + 2];
        for k in 1..MAX_ORDER + 2 {
            gamma[k] = gamma[k - 1] + 1.0 / k as f64;
        }
        let mut alpha = [0.0; MAX_ORDER + 2];
        let mut error_const = [0.0; MAX_ORDER + 2];
        for k in 0..MAX_ORDER + 2 {
            alpha[k] = (1.0 - kappa[k]) * gamma[k];
            error_const[k] = kappa[k] * gamma[k] + 1.0 / (k as f64 + 1.0);
        }
        let mut d = vec![vec![0.0; n]; MAX_ORDER + 3];
        d[0].copy_from_slice(y0);
        for (v, fv) in d[1].iter_mut().zip(f0) {
            *v = signed(dir, h0) * fv;
        }
        let mut s = Self {
            n,
            dir,
            t: t0,
            t_bound,
            h_abs: h0,
            hmax,
            d,
            order: 1,
            max_order,
            n_equal_steps: 0,
            gamma,
            alpha,
            error_const,
            rtol,
            atol,
            newton_tol: (10.0 * f64::EPSILON / rtol).max(rtol.sqrt().min(0.03)),
            jac_fn,
            jac: Mat::zeros(n, n),
            jac_current: false,
            lu: None,
            nonfinite: false,
        };
        s.update_jacobian(f, t0, y0, Some(f0), stats);
        s
    }

    fn update_jacobian(
        &mut self,
        f: &mut Rhs<'_>,
        t: f64,
        y: &[f64],
        fy: Option<&[f64]>,
        stats: &mut OdeStats,
    ) {
        let n = self.n;
        stats.jac_evals += 1;
        let mut jac = Mat::zeros(n, n);
        if let Some(jf) = self.jac_fn.as_mut() {
            let mut m = Matrix::zeros(n, n);
            jf(t, y, &mut m);
            jac = Mat::from_fn(n, n, |i, j| m.get(i, j));
        } else {
            // Diferenças progressivas com incremento √(ε·max(1e-5, |yⱼ|)) (Hairer & Wanner).
            let f0 = match fy {
                Some(v) => v.to_vec(),
                None => {
                    let mut v = vec![0.0; n];
                    f(t, y, &mut v);
                    stats.fun_evals += 1;
                    v
                }
            };
            let mut yp = y.to_vec();
            let mut fp = vec![0.0; n];
            for j in 0..n {
                let delta = (f64::EPSILON * y[j].abs().max(1e-5)).sqrt();
                yp[j] = y[j] + delta;
                let dj = yp[j] - y[j];
                f(t, &yp, &mut fp);
                stats.fun_evals += 1;
                for i in 0..n {
                    jac[(i, j)] = (fp[i] - f0[i]) / dj;
                }
                yp[j] = y[j];
            }
        }
        // Um Jacobiano com NaN ou ±∞ (f não finito perto de t) não substitui o atual: o passo
        // falha e é reduzido, e o Jacobiano finito anterior continua em uso.
        if (0..n).all(|j| (0..n).all(|i| jac[(i, j)].is_finite())) {
            self.jac = jac;
            self.lu = None;
        } else {
            self.nonfinite = true;
        }
        self.jac_current = true;
    }

    /// Reescreve D₀..D_ordem para o passo h·factor (Shampine & Reichelt, §2.3).
    fn change_d(&mut self, factor: f64) {
        let k = self.order;
        let r = compute_r(k, factor);
        let u = compute_r(k, 1.0);
        // RU = R·U; D_novo[i] = Σⱼ RU[j][i]·D[j].
        let mut ru = vec![vec![0.0; k + 1]; k + 1];
        for i in 0..=k {
            for j in 0..=k {
                ru[i][j] = (0..=k).map(|m| r[i][m] * u[m][j]).sum();
            }
        }
        let old: Vec<Vec<f64>> = self.d[..=k].to_vec();
        for i in 0..=k {
            for c in 0..self.n {
                self.d[i][c] = (0..=k).map(|j| ru[j][i] * old[j][c]).sum();
            }
        }
        self.h_abs *= factor;
        self.n_equal_steps = 0;
        self.lu = None;
    }

    fn rms_scaled(&self, v: &[f64], y: &[f64]) -> f64 {
        let s: f64 = v
            .iter()
            .zip(y)
            .zip(&self.atol)
            .map(|((e, yv), a)| {
                let sc = a + self.rtol * yv.abs();
                if sc == 0.0 {
                    if *e == 0.0 { 0.0 } else { f64::INFINITY }
                } else {
                    (e / sc).powi(2)
                }
            })
            .sum();
        (s / self.n as f64).sqrt()
    }
}

/// R(k, ρ): R₀ⱼ = 1 e Rᵢⱼ = Π_{m=1..i} (m − 1 − ρj)/m.
fn compute_r(k: usize, factor: f64) -> Vec<Vec<f64>> {
    let mut r = vec![vec![0.0; k + 1]; k + 1];
    r[0].iter_mut().for_each(|v| *v = 1.0);
    for i in 1..=k {
        for j in 0..=k {
            r[i][j] = r[i - 1][j] * (i as f64 - 1.0 - factor * j as f64) / i as f64;
        }
    }
    r
}

impl Stepper for Bdf<'_, '_> {
    fn t(&self) -> f64 {
        self.t
    }

    fn y(&self) -> &[f64] {
        &self.d[0]
    }

    fn step(&mut self, f: &mut Rhs<'_>, stats: &mut OdeStats) -> Outcome {
        let n = self.n;
        let mut y_predict = vec![0.0; n];
        let mut psi = vec![0.0; n];
        let mut y_new = vec![0.0; n];
        let mut dsum = vec![0.0; n];
        let mut fy = vec![0.0; n];
        let mut rhs = vec![0.0; n];
        let (t_new, n_iter) = loop {
            let hmin = 16.0 * f64::EPSILON * self.t.abs().max(f64::MIN_POSITIVE);
            if self.h_abs > self.hmax {
                self.change_d(self.hmax / self.h_abs);
            }
            if self.h_abs < hmin {
                return Outcome::Failed(if self.nonfinite {
                    OdeStatus::NonFinite
                } else {
                    OdeStatus::StepTooSmall
                });
            }
            let mut t_new = self.t + signed(self.dir, self.h_abs);
            if signed(self.dir, t_new - self.t_bound) > 0.0 {
                t_new = self.t_bound;
                self.change_d((t_new - self.t).abs() / self.h_abs);
            }
            let h = t_new - self.t;
            let k = self.order;
            for c in 0..n {
                y_predict[c] = (0..=k).map(|i| self.d[i][c]).sum();
                psi[c] = (1..=k).map(|i| self.d[i][c] * self.gamma[i]).sum::<f64>() / self.alpha[k];
            }
            let cfac = h / self.alpha[k];
            // Newton simplificado; com Jacobiano velho, uma falha pede Jacobiano novo antes de
            // reduzir o passo.
            let outcome = loop {
                if self.lu.is_none() {
                    let m = Mat::from_fn(n, n, |i, j| {
                        (if i == j { 1.0 } else { 0.0 }) - cfac * self.jac[(i, j)]
                    });
                    let lu = m.partial_piv_lu();
                    stats.lu_decomps += 1;
                    let u = lu.U();
                    if (0..n).any(|i| u[(i, i)] == 0.0 || !u[(i, i)].is_finite()) {
                        break None;
                    }
                    self.lu = Some(lu);
                }
                let lu = self.lu.as_ref().expect("LU calculada acima");
                y_new.copy_from_slice(&y_predict);
                dsum.iter_mut().for_each(|v| *v = 0.0);
                let mut rate: Option<f64> = None;
                let mut dy_norm_old: Option<f64> = None;
                let mut converged = false;
                let mut iters = 0;
                for it in 0..NEWTON_MAXITER {
                    iters = it + 1;
                    f(t_new, &y_new, &mut fy);
                    stats.fun_evals += 1;
                    if fy.iter().any(|v| !v.is_finite()) {
                        self.nonfinite = true;
                        break;
                    }
                    for c in 0..n {
                        rhs[c] = cfac * fy[c] - psi[c] - dsum[c];
                    }
                    let residual_zero = rhs.iter().all(|v| *v == 0.0);
                    lu.solve_in_place(MatMut::from_column_major_slice_mut(&mut rhs, n, 1));
                    let dy_norm = self.rms_scaled(&rhs, &y_new);
                    // dy = 0 com resíduo não nulo: a matriz de iteração é inútil (um Jacobiano
                    // absurdo faz o passo de Newton sumir no arredondamento). Conta como falha,
                    // em vez de aceitar o preditor como solução (regressão).
                    if dy_norm == 0.0 && !residual_zero {
                        break;
                    }
                    if let Some(old) = dy_norm_old {
                        rate = Some(dy_norm / old);
                    }
                    if let Some(r) = rate
                        && (r >= 1.0
                            || r.powi((NEWTON_MAXITER - it) as i32) / (1.0 - r) * dy_norm > self.newton_tol)
                    {
                        break;
                    }
                    for c in 0..n {
                        y_new[c] += rhs[c];
                        dsum[c] += rhs[c];
                    }
                    if dy_norm == 0.0 || rate.is_some_and(|r| r / (1.0 - r) * dy_norm < self.newton_tol) {
                        converged = true;
                        break;
                    }
                    dy_norm_old = Some(dy_norm);
                }
                if converged {
                    break Some(iters);
                }
                if self.jac_current {
                    break None;
                }
                self.update_jacobian(f, t_new, &y_predict, None, stats);
            };
            let Some(n_iter) = outcome else {
                stats.rejected += 1;
                self.change_d(0.5);
                continue;
            };
            self.nonfinite = false;
            let safety = 0.9 * (2 * NEWTON_MAXITER + 1) as f64 / (2 * NEWTON_MAXITER + n_iter) as f64;
            let err: Vec<f64> = dsum.iter().map(|v| self.error_const[k] * v).collect();
            let en = self.rms_scaled(&err, &y_new);
            if en > 1.0 {
                stats.rejected += 1;
                let factor = (safety * en.powf(-1.0 / (k as f64 + 1.0))).max(MIN_FACTOR);
                self.change_d(factor);
                continue;
            }
            break (t_new, n_iter);
        };
        // Passo aceito: atualiza as diferenças.
        stats.steps += 1;
        let k = self.order;
        self.n_equal_steps += 1;
        self.t = t_new;
        for c in 0..n {
            self.d[k + 2][c] = dsum[c] - self.d[k + 1][c];
            self.d[k + 1][c] = dsum[c];
        }
        for i in (0..=k).rev() {
            for c in 0..n {
                self.d[i][c] += self.d[i + 1][c];
            }
        }
        self.jac_current = false;
        if self.n_equal_steps > k {
            // Escolha de ordem: k − 1, k ou k + 1, pela maior razão de passo admissível.
            let y = self.d[0].clone();
            let norm_of = |s: &Self, i: usize, kk: usize| {
                let v: Vec<f64> = s.d[i].iter().map(|x| s.error_const[kk] * x).collect();
                s.rms_scaled(&v, &y)
            };
            let em = if k > 1 {
                norm_of(self, k, k - 1)
            } else {
                f64::INFINITY
            };
            let ek = {
                let v: Vec<f64> = dsum.iter().map(|x| self.error_const[k] * x).collect();
                self.rms_scaled(&v, &y)
            };
            let ep = if k < self.max_order {
                norm_of(self, k + 2, k + 1)
            } else {
                f64::INFINITY
            };
            let factors = [
                em.powf(-1.0 / k as f64),
                ek.powf(-1.0 / (k as f64 + 1.0)),
                ep.powf(-1.0 / (k as f64 + 2.0)),
            ];
            let mut best = 1;
            for (i, v) in factors.iter().enumerate() {
                if *v > factors[best] {
                    best = i;
                }
            }
            self.order = k + best - 1;
            let safety = 0.9 * (2 * NEWTON_MAXITER + 1) as f64 / (2 * NEWTON_MAXITER + n_iter) as f64;
            let factor = (safety * factors[best]).min(MAX_FACTOR);
            self.change_d(factor);
        }
        let k = self.order;
        Outcome::Accepted(Interp::Bdf {
            t: self.t,
            h: signed(self.dir, self.h_abs),
            order: k,
            d: self.d[..=k].iter().flatten().copied().collect(),
        })
    }
}
