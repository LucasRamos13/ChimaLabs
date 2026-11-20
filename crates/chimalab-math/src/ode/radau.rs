//! Radau IIA de 3 estágios (ordem 5, L-estável, rigidamente preciso), com matriz de massa: resolve
//! M y' = f(t, y), inclusive com M singular (DAE de índice 1).
//!
//! Fonte: E. Hairer & G. Wanner, *Solving Ordinary Differential Equations II*, 2.ª ed. (1996):
//! - §IV.5, para o método;
//! - §IV.8, para a implementação:
//!   - Newton simplificado desacoplado pela decomposição A⁻¹ = TΛT⁻¹, que dá um sistema real n×n e um complexo, aqui resolvido como real 2n×2n;
//!   - estimador embutido filtrado por (γ/h·M − J)⁻¹;
//!   - controle de passo preditivo de Gustafsson;
//!   - valores iniciais extrapolados do polinômio de colocação anterior;
//!   - transformação das tolerâncias rtol' = 0,1·rtol^{2/3}.
//!
//! As constantes vêm de `ferramentas/oraculos/f049_radau.py`, que as deriva só dos nós, a 50 dígitos, e confere as condições de ordem 5 do método e de ordem 3 do embutido.

use faer::linalg::solvers::{PartialPivLu, Solve};
use faer::{Mat, MatMut};

use super::{Interp, JacFn, OdeStats, OdeStatus, Outcome, Rhs, Stepper, signed};
use crate::linalg::Matrix;

const GAMMA: f64 = 3.637_834_252_744_496;
const ALPHA: f64 = 2.681_082_873_627_752;
const BETA: f64 = 3.050_430_199_247_410_6;
const T: [[f64; 3]; 3] = [
    [
        0.025_739_097_219_432_99,
        0.023_459_807_669_881_926,
        0.157_154_699_506_071_9,
    ],
    [
        0.068_195_089_895_788_18,
        -0.406_802_844_843_365_74,
        -0.249_987_798_979_195_2,
    ],
    [
        0.272_548_014_618_867_6,
        0.067_390_058_400_265_57,
        -1.098_231_612_051_077_6,
    ],
];
const TINV: [[f64; 3]; 3] = [
    [
        15.332_045_611_836_3,
        1.202_293_919_547_700_8,
        1.920_309_147_110_616_7,
    ],
    [
        0.223_570_494_679_697_17,
        -2.351_335_579_260_006,
        0.567_221_297_579_548_5,
    ],
    [
        3.818_670_828_835_964_4,
        0.154_089_699_202_938_05,
        -0.399_185_812_282_373_8,
    ],
];
pub(crate) const C: [f64; 3] = [0.155_051_025_721_682_2, 0.644_948_974_278_317_8, 1.0];
const E: [f64; 3] = [
    -2.762_305_454_748_599_4,
    0.379_935_598_252_728_9,
    -0.091_629_609_865_225_8,
];
const NEWTON_MAXITER: usize = 7;

pub(crate) struct Radau<'j, 'a> {
    n: usize,
    dir: f64,
    t: f64,
    y: Vec<f64>,
    f0: Vec<f64>,
    t_bound: f64,
    h_abs: f64,
    hmax: f64,
    rtol: f64,
    atol: Vec<f64>,
    newton_tol: f64,
    mass: Option<Mat<f64>>,
    jac_fn: Option<&'j mut JacFn<'a>>,
    jac: Mat<f64>,
    jac_current: bool,
    need_jac: bool,
    lu_h: f64,
    lu1: Option<PartialPivLu<f64>>,
    lu2: Option<PartialPivLu<f64>>,
    first: bool,
    rejected_last: bool,
    nonfinite: bool,
    h_acc: f64,
    err_acc: f64,
    eta_old: f64,
    /// Polinômio de colocação do último passo aceito (para os valores iniciais).
    prev: Option<Interp>,
}

impl<'j, 'a> Radau<'j, 'a> {
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
        mass: Option<&Matrix>,
        jac_fn: Option<&'j mut JacFn<'a>>,
        stats: &mut OdeStats,
    ) -> Self {
        let n = y0.len();
        // Transformação das tolerâncias de Hairer & Wanner: o erro de um método de ordem 5
        // controlado por passo fica proporcional à tolerância pedida.
        let rtol_i = 0.1 * rtol.powf(2.0 / 3.0);
        let atol_i: Vec<f64> = atol.iter().map(|a| a * rtol_i / rtol).collect();
        let mut s = Self {
            n,
            dir: if t_bound > t0 { 1.0 } else { -1.0 },
            t: t0,
            y: y0.to_vec(),
            f0: f0.to_vec(),
            t_bound,
            h_abs: h0,
            hmax,
            rtol: rtol_i,
            atol: atol_i,
            newton_tol: (10.0 * f64::EPSILON / rtol_i).max(rtol_i.sqrt().min(0.03)),
            mass: mass.map(|m| Mat::from_fn(n, n, |i, j| m.get(i, j))),
            jac_fn,
            jac: Mat::zeros(n, n),
            jac_current: false,
            need_jac: true,
            lu_h: f64::NAN,
            lu1: None,
            lu2: None,
            first: true,
            rejected_last: false,
            nonfinite: false,
            h_acc: h0,
            err_acc: 1e-2,
            eta_old: 1.0,
            prev: None,
        };
        s.update_jacobian(f, stats);
        s
    }

    fn update_jacobian(&mut self, f: &mut Rhs<'_>, stats: &mut OdeStats) {
        let n = self.n;
        stats.jac_evals += 1;
        let mut jac = Mat::zeros(n, n);
        if let Some(jf) = self.jac_fn.as_mut() {
            let mut m = Matrix::zeros(n, n);
            jf(self.t, &self.y, &mut m);
            jac = Mat::from_fn(n, n, |i, j| m.get(i, j));
        } else {
            let mut yp = self.y.clone();
            let mut fp = vec![0.0; n];
            for j in 0..n {
                let delta = (f64::EPSILON * self.y[j].abs().max(1e-5)).sqrt();
                yp[j] = self.y[j] + delta;
                let dj = yp[j] - self.y[j];
                f(self.t, &yp, &mut fp);
                stats.fun_evals += 1;
                for i in 0..n {
                    jac[(i, j)] = (fp[i] - self.f0[i]) / dj;
                }
                yp[j] = self.y[j];
            }
        }
        if (0..n).all(|j| (0..n).all(|i| jac[(i, j)].is_finite())) {
            self.jac = jac;
        } else {
            self.nonfinite = true;
        }
        self.jac_current = true;
        self.need_jac = false;
        self.lu1 = None;
        self.lu2 = None;
    }

    fn m(&self, i: usize, j: usize) -> f64 {
        match &self.mass {
            Some(m) => m[(i, j)],
            None => f64::from(u8::from(i == j)),
        }
    }

    /// M·v.
    fn mul_m(&self, v: &[f64], out: &mut [f64]) {
        match &self.mass {
            Some(m) => {
                for (i, o) in out.iter_mut().enumerate() {
                    *o = (0..self.n).map(|j| m[(i, j)] * v[j]).sum();
                }
            }
            None => out.copy_from_slice(v),
        }
    }

    /// Fatora E₁ = γ/h·M − J e E₂ = [[α/h·M − J, β/h·M], [−β/h·M, α/h·M − J]]. False se singular.
    fn factor(&mut self, h: f64, stats: &mut OdeStats) -> bool {
        let n = self.n;
        let (g, a, b) = (GAMMA / h, ALPHA / h, BETA / h);
        let e1 = Mat::from_fn(n, n, |i, j| g * self.m(i, j) - self.jac[(i, j)]);
        let e2 = Mat::from_fn(2 * n, 2 * n, |i, j| {
            let (bi, ii) = (i / n, i % n);
            let (bj, jj) = (j / n, j % n);
            let mij = self.m(ii, jj);
            match (bi, bj) {
                (0, 0) | (1, 1) => a * mij - self.jac[(ii, jj)],
                (0, 1) => b * mij,
                _ => -b * mij,
            }
        });
        let l1 = e1.partial_piv_lu();
        let l2 = e2.partial_piv_lu();
        stats.lu_decomps += 2;
        let ok = |u: faer::MatRef<'_, f64>| (0..u.nrows()).all(|i| u[(i, i)] != 0.0 && u[(i, i)].is_finite());
        if !(ok(l1.U()) && ok(l2.U())) {
            return false;
        }
        self.lu1 = Some(l1);
        self.lu2 = Some(l2);
        self.lu_h = h;
        true
    }

    fn norm(&self, v: &[f64], y_new: Option<&[f64]>) -> f64 {
        let n = self.n;
        let k = v.len() / n;
        let s: f64 = v
            .iter()
            .enumerate()
            .map(|(idx, e)| {
                let i = idx % n;
                let ya = match y_new {
                    Some(yn) => self.y[i].abs().max(yn[i].abs()),
                    None => self.y[i].abs(),
                };
                let sc = self.atol[i] + self.rtol * ya;
                if sc == 0.0 {
                    if *e == 0.0 { 0.0 } else { f64::INFINITY }
                } else {
                    (e / sc).powi(2)
                }
            })
            .sum();
        (s / (k * n) as f64).sqrt()
    }
}

/// Polinômio de colocação nos nós 0, c₁, c₂, 1: y(t + θh) = y + Σ ℓᵢ(θ)·zᵢ (ℓᵢ de Lagrange).
pub(crate) fn collocation_eval(y: &[f64], z: &[f64], theta: f64, out: &mut [f64]) {
    let n = y.len();
    let nodes = [0.0, C[0], C[1], C[2]];
    let mut l = [0.0; 3];
    for (k, lk) in l.iter_mut().enumerate() {
        let xk = nodes[k + 1];
        let mut p = 1.0;
        for (m, xm) in nodes.iter().enumerate() {
            if m != k + 1 {
                p *= (theta - xm) / (xk - xm);
            }
        }
        *lk = p;
    }
    for (i, o) in out.iter_mut().enumerate() {
        *o = y[i] + l[0] * z[i] + l[1] * z[n + i] + l[2] * z[2 * n + i];
    }
}

impl Stepper for Radau<'_, '_> {
    fn t(&self) -> f64 {
        self.t
    }

    fn y(&self) -> &[f64] {
        &self.y
    }

    fn step(&mut self, f: &mut Rhs<'_>, stats: &mut OdeStats) -> Outcome {
        let n = self.n;
        let mut z = vec![0.0; 3 * n];
        let mut w = vec![0.0; 3 * n];
        let mut fz = vec![0.0; 3 * n];
        let mut ytmp = vec![0.0; n];
        let mut mw = vec![0.0; n];
        loop {
            if self.need_jac {
                self.update_jacobian(f, stats);
            }
            let hmin = 16.0 * f64::EPSILON * self.t.abs().max(f64::MIN_POSITIVE);
            self.h_abs = self.h_abs.min(self.hmax);
            let mut h = self.h_abs;
            let mut last = false;
            if signed(self.dir, self.t + signed(self.dir, h * 1.01) - self.t_bound) >= 0.0 {
                h = (self.t_bound - self.t).abs();
                last = true;
            }
            if h < hmin {
                return Outcome::Failed(if self.nonfinite {
                    OdeStatus::NonFinite
                } else {
                    OdeStatus::StepTooSmall
                });
            }
            let hs = signed(self.dir, h);
            if (self.lu1.is_none() || self.lu_h != hs) && !self.factor(hs, stats) {
                stats.rejected += 1;
                self.h_abs *= 0.5;
                self.rejected_last = true;
                continue;
            }
            // Valores iniciais: extrapolação do polinômio de colocação anterior.
            match &self.prev {
                Some(Interp::Radau { t, h: hp, y, z: zp }) if !self.first => {
                    for k in 0..3 {
                        let theta = (self.t + C[k] * hs - t) / hp;
                        collocation_eval(y, zp, theta, &mut ytmp);
                        for i in 0..n {
                            z[k * n + i] = ytmp[i] - self.y[i];
                        }
                    }
                }
                _ => z.iter_mut().for_each(|v| *v = 0.0),
            }
            for k in 0..3 {
                for i in 0..n {
                    w[k * n + i] = (0..3).map(|j| TINV[k][j] * z[j * n + i]).sum();
                }
            }
            // Newton simplificado.
            let (g, a, b) = (GAMMA / hs, ALPHA / hs, BETA / hs);
            let mut converged = false;
            let mut dw_old = 0.0;
            let mut eta = self.eta_old.max(f64::EPSILON).powf(0.8);
            let mut theta = 0.0f64;
            let mut iters = 0;
            for it in 0..NEWTON_MAXITER {
                iters = it + 1;
                let mut finite = true;
                for k in 0..3 {
                    for i in 0..n {
                        ytmp[i] = self.y[i] + z[k * n + i];
                    }
                    f(self.t + C[k] * hs, &ytmp, &mut fz[k * n..(k + 1) * n]);
                    stats.fun_evals += 1;
                    finite &= fz[k * n..(k + 1) * n].iter().all(|v| v.is_finite());
                }
                if !finite {
                    self.nonfinite = true;
                    break;
                }
                // Resíduos transformados: G = T⁻¹F − (Λ/h ⊗ M)W.
                let mut r1 = vec![0.0; n];
                let mut r23 = vec![0.0; 2 * n];
                let gk = |k: usize, i: usize| -> f64 { (0..3).map(|j| TINV[k][j] * fz[j * n + i]).sum() };
                self.mul_m(&w[..n], &mut mw);
                for i in 0..n {
                    r1[i] = gk(0, i) - g * mw[i];
                }
                let mut mw2 = vec![0.0; n];
                let mut mw3 = vec![0.0; n];
                self.mul_m(&w[n..2 * n], &mut mw2);
                self.mul_m(&w[2 * n..], &mut mw3);
                for i in 0..n {
                    r23[i] = gk(1, i) - a * mw2[i] - b * mw3[i];
                    r23[n + i] = gk(2, i) + b * mw2[i] - a * mw3[i];
                }
                let resid_zero = r1.iter().chain(&r23).all(|v| *v == 0.0);
                let lu1 = self.lu1.as_ref().expect("fatorada acima");
                let lu2 = self.lu2.as_ref().expect("fatorada acima");
                lu1.solve_in_place(MatMut::from_column_major_slice_mut(&mut r1, n, 1));
                lu2.solve_in_place(MatMut::from_column_major_slice_mut(&mut r23, 2 * n, 1));
                let dw: Vec<f64> = r1.iter().chain(&r23).copied().collect();
                let dw_norm = self.norm(&dw, None);
                if dw_norm == 0.0 && !resid_zero {
                    break;
                }
                if it > 0 {
                    theta = dw_norm / dw_old;
                    if theta >= 0.99 {
                        break;
                    }
                    eta = theta / (1.0 - theta);
                    let restante = (NEWTON_MAXITER - 1 - it) as i32;
                    if theta.powi(restante) / (1.0 - theta) * dw_norm > self.newton_tol {
                        break;
                    }
                }
                dw_old = dw_norm.max(f64::MIN_POSITIVE);
                for (wi, d) in w.iter_mut().zip(&dw) {
                    *wi += d;
                }
                for k in 0..3 {
                    for i in 0..n {
                        z[k * n + i] = (0..3).map(|j| T[k][j] * w[j * n + i]).sum();
                    }
                }
                if eta * dw_norm <= self.newton_tol {
                    converged = true;
                    break;
                }
            }
            if !converged {
                stats.rejected += 1;
                if !self.jac_current {
                    self.need_jac = true;
                    continue;
                }
                self.h_abs *= 0.5;
                self.rejected_last = true;
                self.lu1 = None;
                continue;
            }
            self.nonfinite = false;
            self.eta_old = eta;
            // Erro: (γ/h·M − J)⁻¹ (f₀ + γ/h·M·Σ eᵢzᵢ).
            let v: Vec<f64> = (0..n)
                .map(|i| (0..3).map(|k| E[k] * z[k * n + i]).sum::<f64>() * g)
                .collect();
            let mut mv = vec![0.0; n];
            self.mul_m(&v, &mut mv);
            let y_new: Vec<f64> = (0..n).map(|i| self.y[i] + z[2 * n + i]).collect();
            let lu1 = self.lu1.as_ref().expect("fatorada acima");
            let mut err: Vec<f64> = (0..n).map(|i| self.f0[i] + mv[i]).collect();
            lu1.solve_in_place(MatMut::from_column_major_slice_mut(&mut err, n, 1));
            let mut errn = self.norm(&err, Some(&y_new));
            if errn >= 1.0 && (self.first || self.rejected_last) {
                // Segunda filtragem (Hairer & Wanner): f avaliado em y + err.
                for i in 0..n {
                    ytmp[i] = self.y[i] + err[i];
                }
                let mut f1 = vec![0.0; n];
                f(self.t, &ytmp, &mut f1);
                stats.fun_evals += 1;
                if f1.iter().all(|x| x.is_finite()) {
                    let mut e2: Vec<f64> = (0..n).map(|i| f1[i] + mv[i]).collect();
                    lu1.solve_in_place(MatMut::from_column_major_slice_mut(&mut e2, n, 1));
                    errn = self.norm(&e2, Some(&y_new));
                }
            }
            let errn = errn.max(1e-10);
            let fac = 0.9 * (2 * NEWTON_MAXITER + 1) as f64 / (2 * NEWTON_MAXITER + iters) as f64;
            let mut quot = (errn.powf(0.25) / fac).clamp(0.2, 8.0);
            if errn >= 1.0 {
                stats.rejected += 1;
                self.h_abs = h / quot;
                if self.first {
                    self.h_abs = h * 0.1;
                }
                self.rejected_last = true;
                continue;
            }
            // Passo aceito.
            if !self.first {
                // Controle preditivo de Gustafsson.
                let facgus = (self.h_acc / h) * (errn * errn / self.err_acc).powf(0.25) / fac;
                quot = quot.max(facgus.clamp(0.2, 8.0));
            }
            self.h_acc = h;
            self.err_acc = errn.max(1e-2);
            stats.steps += 1;
            let t_old = self.t;
            let interp = Interp::Radau {
                t: t_old,
                h: hs,
                y: self.y.clone(),
                z: z.clone(),
            };
            self.t = if last { self.t_bound } else { self.t + hs };
            self.y = y_new;
            f(self.t, &self.y, &mut self.f0);
            stats.fun_evals += 1;
            self.first = false;
            let h_new = h / quot;
            let ratio = h_new / h;
            // Jacobiano reaproveitado se o Newton convergiu muito rápido; LU reaproveitada se o
            // passo quase não muda.
            self.need_jac = theta > 1e-3;
            self.jac_current = false;
            if !self.need_jac && (1.0..=1.2).contains(&ratio) {
                self.h_abs = h;
            } else {
                self.h_abs = if self.rejected_last { h_new.min(h) } else { h_new };
                self.lu1 = None;
            }
            self.rejected_last = false;
            self.prev = Some(interp.clone());
            return Outcome::Accepted(interp);
        }
    }
}
