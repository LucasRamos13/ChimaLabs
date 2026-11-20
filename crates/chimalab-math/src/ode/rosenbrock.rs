//! Rosenbrock modificado de ordem 2(3) para problemas rígidos, pouco exigente em precisão
//! (L. F. Shampine & M. W. Reichelt, SIAM J. Sci. Comput. 18 (1997) 1–22, §3).
//!
//! Método W de um passo e L-estável, com d = 1/(2 + √2) e W = I − h·d·J:
//! - k₁ = W⁻¹(F₀ + h·d·T), com T ≈ ∂f/∂t;
//! - F₁ = f(t + h/2, y + h·k₁/2) e k₂ = W⁻¹(F₁ − k₁) + k₁;
//! - y₁ = y + h·k₂ e F₂ = f(t + h, y₁);
//! - k₃ = W⁻¹(F₂ − e₃₂(k₂ − F₁) − 2(k₁ − F₀) + h·d·T), com e₃₂ = 6 + √2;
//! - erro estimado: h(k₁ − 2k₂ + k₃)/6.
//!
//! A saída densa é a do próprio método, y(t + sh) = y + h[s(1 − s)k₁ + s(s − 2d)k₂]/(1 − 2d). Ela
//! entra no formato de [`super::rk::dense_eval`] com r₀ = y, r₁ = h·k₂ e r₂ = h(k₁ − k₂)/(1 − 2d).
//!
//! Uma decomposição LU por passo tentado. O Jacobiano é refeito a cada passo aceito (por
//! diferenças ou pela função dada) e reaproveitado nas rejeições.

use faer::Mat;
use faer::linalg::solvers::{PartialPivLu, Solve};

use super::{Interp, JacFn, OdeStats, OdeStatus, Outcome, Rhs, Stepper, signed};
use crate::linalg::Matrix;

const D: f64 = 0.292_893_218_813_452_5; // 1/(2 + √2)
const E32: f64 = 7.414_213_562_373_095; // 6 + √2

pub(crate) struct Rosenbrock<'j, 'a> {
    n: usize,
    dir: f64,
    t: f64,
    y: Vec<f64>,
    f0: Vec<f64>,
    t_bound: f64,
    h: f64,
    hmax: f64,
    rtol: f64,
    atol: Vec<f64>,
    jac_fn: Option<&'j mut JacFn<'a>>,
    jac: Mat<f64>,
    dfdt: Vec<f64>,
    jac_atual: bool,
    rejected_last: bool,
    nonfinite_last: bool,
    recusas_lin: u32,
}

impl<'j, 'a> Rosenbrock<'j, 'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        t0: f64,
        y0: &[f64],
        f0: &[f64],
        t_bound: f64,
        h0: f64,
        hmax: f64,
        rtol: f64,
        atol: Vec<f64>,
        jac_fn: Option<&'j mut JacFn<'a>>,
    ) -> Self {
        let n = y0.len();
        Self {
            n,
            dir: if t_bound > t0 { 1.0 } else { -1.0 },
            t: t0,
            y: y0.to_vec(),
            f0: f0.to_vec(),
            t_bound,
            h: h0,
            hmax,
            rtol,
            atol,
            jac_fn,
            jac: Mat::zeros(n, n),
            dfdt: vec![0.0; n],
            jac_atual: false,
            rejected_last: false,
            nonfinite_last: false,
            recusas_lin: 0,
        }
    }

    /// J = ∂f/∂y e T = ∂f/∂t em (t, y), com F₀ = f(t, y) já conhecido.
    fn atualiza_jacobiano(&mut self, f: &mut Rhs<'_>, hs: f64, stats: &mut OdeStats) {
        let n = self.n;
        stats.jac_evals += 1;
        let (t, y) = (self.t, &self.y);
        let mut jac = Mat::zeros(n, n);
        if let Some(jf) = self.jac_fn.as_mut() {
            let mut m = Matrix::zeros(n, n);
            jf(t, y, &mut m);
            jac = Mat::from_fn(n, n, |i, j| m.get(i, j));
        } else {
            let mut yp = y.clone();
            let mut fp = vec![0.0; n];
            for j in 0..n {
                let delta = (f64::EPSILON * y[j].abs().max(1e-5)).sqrt();
                yp[j] = y[j] + delta;
                let dj = yp[j] - y[j];
                f(t, &yp, &mut fp);
                stats.fun_evals += 1;
                for i in 0..n {
                    jac[(i, j)] = (fp[i] - self.f0[i]) / dj;
                }
                yp[j] = y[j];
            }
        }
        // Jacobiano não finito não substitui o anterior (como no BDF); o passo falha e é reduzido.
        if (0..n).all(|j| (0..n).all(|i| jac[(i, j)].is_finite())) {
            self.jac = jac;
        }
        // T = ∂f/∂t por diferença no sentido da integração; se f não for finito lá (f definida só
        // até um instante), diferença no sentido oposto. Regressão: com a sonda além de t = 1 em
        // f = √(1 − t), T ficava NaN em todas as tentativas e a integração parava em 1 − 1e-9.
        let mut fp = vec![0.0; n];
        let mut dfdt = vec![f64::NAN; n];
        for sentido in [self.dir, -self.dir] {
            let tp = t + signed(sentido, f64::EPSILON.sqrt() * t.abs().max(hs.abs()));
            f(tp, y, &mut fp);
            stats.fun_evals += 1;
            let dt = tp - t;
            for i in 0..n {
                dfdt[i] = (fp[i] - self.f0[i]) / dt;
            }
            if dfdt.iter().all(|v| v.is_finite()) {
                break;
            }
        }
        if dfdt.iter().all(|v| v.is_finite()) {
            self.dfdt = dfdt;
        }
        self.jac_atual = true;
    }

    fn error_norm(&self, y_new: &[f64], err: &[f64]) -> f64 {
        let s: f64 = (0..self.n)
            .map(|i| {
                let sc = self.atol[i] + self.rtol * self.y[i].abs().max(y_new[i].abs());
                if sc == 0.0 {
                    if err[i] == 0.0 { 0.0 } else { f64::INFINITY }
                } else {
                    (err[i] / sc).powi(2)
                }
            })
            .sum();
        (s / self.n as f64).sqrt()
    }
}

fn resolve(lu: &PartialPivLu<f64>, b: &[f64]) -> Vec<f64> {
    let x = lu.solve(Mat::from_fn(b.len(), 1, |i, _| b[i]));
    (0..b.len()).map(|i| x[(i, 0)]).collect()
}

impl Stepper for Rosenbrock<'_, '_> {
    fn t(&self) -> f64 {
        self.t
    }

    fn y(&self) -> &[f64] {
        &self.y
    }

    fn step(&mut self, f: &mut Rhs<'_>, stats: &mut OdeStats) -> Outcome {
        let n = self.n;
        loop {
            let hmin = 16.0 * f64::EPSILON * self.t.abs().max(f64::MIN_POSITIVE);
            let mut h = self.h.min(self.hmax);
            let mut last = false;
            if signed(self.dir, self.t + signed(self.dir, h * 1.01) - self.t_bound) >= 0.0 {
                h = (self.t_bound - self.t).abs();
                last = true;
            }
            if h < hmin {
                return Outcome::Failed(if self.nonfinite_last {
                    OdeStatus::NonFinite
                } else {
                    OdeStatus::StepTooSmall
                });
            }
            let hs = signed(self.dir, h);
            if !self.jac_atual {
                self.atualiza_jacobiano(f, hs, stats);
            }
            let w = Mat::from_fn(n, n, |i, j| {
                (if i == j { 1.0 } else { 0.0 }) - hs * D * self.jac[(i, j)]
            });
            // h·d·J que estoura (J enorme com passo grande): W⁻¹ levaria tudo a ±0 e o passo
            // "passaria" sem mudar y. Falha do passo, que é reduzido.
            if (0..n).any(|j| (0..n).any(|i| !w[(i, j)].is_finite())) {
                stats.rejected += 1;
                self.rejected_last = true;
                self.h = h * 0.25;
                continue;
            }
            stats.lu_decomps += 1;
            let lu = w.partial_piv_lu();
            let (t, y) = (self.t, &self.y);
            let hdt: Vec<f64> = self.dfdt.iter().map(|v| hs * D * v).collect();
            let b1: Vec<f64> = (0..n).map(|i| self.f0[i] + hdt[i]).collect();
            let k1 = resolve(&lu, &b1);
            let y1: Vec<f64> = (0..n).map(|i| y[i] + 0.5 * hs * k1[i]).collect();
            let mut f1 = vec![0.0; n];
            f(t + 0.5 * hs, &y1, &mut f1);
            let b2: Vec<f64> = (0..n).map(|i| f1[i] - k1[i]).collect();
            let k2: Vec<f64> = resolve(&lu, &b2).iter().zip(&k1).map(|(a, b)| a + b).collect();
            let y_new: Vec<f64> = (0..n).map(|i| y[i] + hs * k2[i]).collect();
            let t_new = if last { self.t_bound } else { t + hs };
            let mut f2 = vec![0.0; n];
            f(t_new, &y_new, &mut f2);
            stats.fun_evals += 2;
            let b3: Vec<f64> = (0..n)
                .map(|i| f2[i] - E32 * (k2[i] - f1[i]) - 2.0 * (k1[i] - self.f0[i]) + hdt[i])
                .collect();
            let k3 = resolve(&lu, &b3);
            let err: Vec<f64> = (0..n).map(|i| hs / 6.0 * (k1[i] - 2.0 * k2[i] + k3[i])).collect();
            let finite = y_new.iter().chain(&f2).chain(&err).all(|v| v.is_finite());
            if !finite {
                stats.rejected += 1;
                self.nonfinite_last = true;
                self.rejected_last = true;
                self.h = h * 0.25;
                continue;
            }
            // Consistência da linearização: F₁ − F₀ = (h/2)(J·k₁ + T) + O(h²). Sem Newton, um
            // Jacobiano grosseiramente errado não seria notado pelo estimador (que também passa
            // por W⁻¹). Regressão: com J = 1e308 em y' = −y, k ≈ 0 e o erro estimado ≈ 0, e o
            // resultado y(1) = 1 saía com estado Completed. Desacordo acima de 90% das
            // variações rejeita o passo; o piso, da ordem do arredondamento de F₁ − F₀, evita
            // rejeições espúrias perto do equilíbrio.
            let mut desacordo = 0.0f64;
            let mut escala = 0.0f64;
            let mut piso = 0.0f64;
            for i in 0..n {
                let jk: f64 = (0..n).map(|j| self.jac[(i, j)] * k1[j]).sum();
                let (dif, lin) = (f1[i] - self.f0[i], 0.5 * hs * (jk + self.dfdt[i]));
                desacordo = desacordo.max((dif - lin).abs());
                escala = escala.max(dif.abs() + lin.abs());
                piso = piso.max(self.f0[i].abs() + f1[i].abs());
            }
            if desacordo > 0.9 * escala + 1e2 * f64::EPSILON * piso {
                // 50 recusas seguidas (h reduzido 4⁵⁰ ≈ 1e30 vezes): Jacobiano incompatível com f.
                // Sem esse limite, com J absurdo e t₀ = 0 (passo mínimo subnormal), passos
                // minúsculos acabariam aceitos e t avançaria sem avançar y, sem fim.
                self.recusas_lin += 1;
                if self.recusas_lin >= 50 {
                    return Outcome::Failed(OdeStatus::StepTooSmall);
                }
                stats.rejected += 1;
                self.nonfinite_last = false;
                self.rejected_last = true;
                self.h = h * 0.25;
                continue;
            }
            self.recusas_lin = 0;
            self.nonfinite_last = false;
            let en = self.error_norm(&y_new, &err);
            if en > 1.0 {
                stats.rejected += 1;
                self.rejected_last = true;
                self.h = h * (0.8 * en.powf(-1.0 / 3.0)).clamp(0.1, 1.0);
                continue;
            }
            stats.steps += 1;
            let mut coeffs = vec![0.0; 5 * n];
            for i in 0..n {
                coeffs[i] = y[i];
                coeffs[n + i] = hs * k2[i];
                coeffs[2 * n + i] = hs * (k1[i] - k2[i]) / (1.0 - 2.0 * D);
            }
            let fac = if en == 0.0 {
                5.0
            } else {
                (0.8 * en.powf(-1.0 / 3.0)).clamp(0.1, 5.0)
            };
            self.h = h * if self.rejected_last { fac.min(1.0) } else { fac };
            self.rejected_last = false;
            self.t = t_new;
            self.y = y_new;
            self.f0 = f2;
            self.jac_atual = false;
            return Outcome::Accepted(Interp::Rk { t, h: hs, coeffs });
        }
    }
}
