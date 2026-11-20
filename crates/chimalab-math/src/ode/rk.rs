//! Tabelas de Runge–Kutta explícitas com estimativa de erro embutida e saída densa.
//!
//! As tabelas são conferidas pelas condições de ordem de Butcher, em aritmética racional exata,
//! por `ferramentas/oraculos/f049_tabelas_rk.py`.

use super::{Interp, OdeStats, OdeStatus, Outcome, Rhs, Stepper, signed};

/// Par explícito com o último estágio igual a f(t + h, y_novo) (FSAL): a última linha de `a`
/// são os pesos b da solução.
pub(crate) struct Tableau {
    /// Nós cᵢ.
    pub c: &'static [f64],
    /// Coeficientes aᵢⱼ, linha a linha (só a parte estritamente inferior).
    pub a: &'static [&'static [f64]],
    /// Pesos do erro b − b̂.
    pub e: &'static [f64],
    /// Ordem da solução (usada no passo inicial).
    pub order: usize,
    /// Ordem do estimador embutido (o expoente do controle de passo é 1/(q + 1)).
    pub err_order: usize,
    /// Pesos do termo de quarta ordem da saída densa (None: Hermite cúbica).
    pub dense: Option<&'static [f64]>,
    /// β do controlador PI.
    pub beta: f64,
    /// Pontos de saída por passo (padrão de `refine`).
    pub refine: usize,
}

/// Bogacki & Shampine, Appl. Math. Lett. 2 (1989) 321–325: ordem 3 com estimador de ordem 2.
pub(crate) static BS23: Tableau = Tableau {
    c: &[0.0, 0.5, 0.75, 1.0],
    a: &[&[], &[0.5], &[0.0, 0.75], &[2.0 / 9.0, 1.0 / 3.0, 4.0 / 9.0]],
    e: &[-5.0 / 72.0, 1.0 / 12.0, 1.0 / 9.0, -1.0 / 8.0],
    order: 3,
    err_order: 2,
    dense: None,
    beta: 0.0,
    refine: 1,
};

/// Dormand & Prince, J. Comput. Appl. Math. 6 (1980) 19–26: ordem 5 com estimador de ordem 4;
/// saída densa de ordem 4 (Hairer, Nørsett & Wanner, *Solving ODE I*, §II.6).
pub(crate) static DOPRI5: Tableau = Tableau {
    c: &[0.0, 0.2, 0.3, 0.8, 8.0 / 9.0, 1.0, 1.0],
    a: &[
        &[],
        &[0.2],
        &[3.0 / 40.0, 9.0 / 40.0],
        &[44.0 / 45.0, -56.0 / 15.0, 32.0 / 9.0],
        &[
            19372.0 / 6561.0,
            -25360.0 / 2187.0,
            64448.0 / 6561.0,
            -212.0 / 729.0,
        ],
        &[
            9017.0 / 3168.0,
            -355.0 / 33.0,
            46732.0 / 5247.0,
            49.0 / 176.0,
            -5103.0 / 18656.0,
        ],
        &[
            35.0 / 384.0,
            0.0,
            500.0 / 1113.0,
            125.0 / 192.0,
            -2187.0 / 6784.0,
            11.0 / 84.0,
        ],
    ],
    e: &[
        71.0 / 57600.0,
        0.0,
        -71.0 / 16695.0,
        71.0 / 1920.0,
        -17253.0 / 339200.0,
        22.0 / 525.0,
        -1.0 / 40.0,
    ],
    order: 5,
    err_order: 4,
    dense: Some(&[
        -12715105075.0 / 11282082432.0,
        0.0,
        87487479700.0 / 32700410799.0,
        -10690763975.0 / 1880347072.0,
        701980252875.0 / 199316789632.0,
        -1453857185.0 / 822651844.0,
        69997945.0 / 29380423.0,
    ]),
    beta: 0.04,
    refine: 4,
};

/// Um passo: preenche `k` (estágios; k[0] = f(t, y) já dado), `y_new` e `err` (= h Σ eⱼkⱼ).
/// Devolve o número de avaliações de f.
#[allow(clippy::too_many_arguments)]
pub(crate) fn step(
    tab: &Tableau,
    f: &mut Rhs<'_>,
    t: f64,
    y: &[f64],
    h: f64,
    k: &mut [Vec<f64>],
    y_new: &mut [f64],
    err: &mut [f64],
) -> usize {
    let n = y.len();
    let s = tab.c.len();
    let mut ys = vec![0.0; n];
    for i in 1..s {
        for (m, v) in ys.iter_mut().enumerate() {
            let mut acc = 0.0;
            for (j, aij) in tab.a[i].iter().enumerate() {
                acc += aij * k[j][m];
            }
            *v = y[m] + h * acc;
        }
        f(t + tab.c[i] * h, &ys, &mut k[i]);
    }
    // A última linha de a é b (FSAL): o argumento do último estágio é a solução.
    y_new.copy_from_slice(&ys);
    for (m, v) in err.iter_mut().enumerate() {
        *v = h * tab.e.iter().enumerate().map(|(j, ej)| ej * k[j][m]).sum::<f64>();
    }
    s - 1
}

/// Coeficientes da saída densa do passo [t, t + h]: r1..r5 concatenados (5n).
pub(crate) fn dense_coeffs(tab: &Tableau, y: &[f64], y_new: &[f64], k: &[Vec<f64>], h: f64) -> Vec<f64> {
    let n = y.len();
    let s = tab.c.len();
    let mut r = vec![0.0; 5 * n];
    for m in 0..n {
        let r2 = y_new[m] - y[m];
        let r3 = h * k[0][m] - r2;
        let r4 = r2 - h * k[s - 1][m] - r3;
        let r5 = match tab.dense {
            Some(d) => h * d.iter().enumerate().map(|(j, dj)| dj * k[j][m]).sum::<f64>(),
            None => 0.0,
        };
        r[m] = y[m];
        r[n + m] = r2;
        r[2 * n + m] = r3;
        r[3 * n + m] = r4;
        r[4 * n + m] = r5;
    }
    r
}

/// y(t + θh) = r1 + θ(r2 + (1 − θ)(r3 + θ(r4 + (1 − θ) r5))).
pub(crate) fn dense_eval(r: &[f64], theta: f64, out: &mut [f64]) {
    let n = out.len();
    let u = 1.0 - theta;
    for (m, o) in out.iter_mut().enumerate() {
        *o = r[m] + theta * (r[n + m] + u * (r[2 * n + m] + theta * (r[3 * n + m] + u * r[4 * n + m])));
    }
}

/// Integrador de passo variável com um par explícito: controle PI do passo (Gustafsson; Hairer,
/// Nørsett & Wanner, vol. II, §IV.2), rejeição de passos com f não finito.
pub(crate) struct RkStepper {
    tab: &'static Tableau,
    t: f64,
    y: Vec<f64>,
    /// Módulo do próximo passo.
    h: f64,
    dir: f64,
    t_bound: f64,
    hmax: f64,
    rtol: f64,
    atol: Vec<f64>,
    k: Vec<Vec<f64>>,
    y_new: Vec<f64>,
    err: Vec<f64>,
    expo: f64,
    facold: f64,
    rejected_last: bool,
    nonfinite_last: bool,
    /// Detecção de rigidez (Hairer, Nørsett & Wanner, vol. I, §IV.2, no código DOPRI5):
    /// contagens de passos rígidos e não rígidos seguidos, e o instante da detecção.
    rigidos: u32,
    nao_rigidos: u32,
    rigidez: Option<f64>,
}

impl RkStepper {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        tab: &'static Tableau,
        t0: f64,
        y0: &[f64],
        f0: &[f64],
        t_bound: f64,
        h0: f64,
        hmax: f64,
        rtol: f64,
        atol: Vec<f64>,
    ) -> Self {
        let n = y0.len();
        let s = tab.c.len();
        let mut k = vec![vec![0.0; n]; s];
        k[0].copy_from_slice(f0);
        Self {
            tab,
            t: t0,
            y: y0.to_vec(),
            h: h0,
            dir: if t_bound > t0 { 1.0 } else { -1.0 },
            t_bound,
            hmax,
            rtol,
            atol,
            k,
            y_new: vec![0.0; n],
            err: vec![0.0; n],
            expo: 1.0 / (tab.err_order as f64 + 1.0) - 0.75 * tab.beta,
            facold: 1e-4,
            rejected_last: false,
            nonfinite_last: false,
            rigidos: 0,
            nao_rigidos: 0,
            rigidez: None,
        }
    }

    /// Teste de rigidez nos métodos com os dois últimos estágios no mesmo ponto (c = 1, como no
    /// DOPRI5): h·ρ ≈ h·‖k_s − k_{s−1}‖/‖y₁ − y_{s−1}‖, com y_{s−1} o argumento do penúltimo
    /// estágio. Acima de 3,25 (borda da região de estabilidade) em 15 passos seguidos, avisa;
    /// 6 passos abaixo zeram a contagem.
    fn testa_rigidez(&mut self, hs: f64) {
        let s = self.tab.c.len();
        if s < 3 || self.tab.c[s - 1] != 1.0 || self.tab.c[s - 2] != 1.0 || self.rigidez.is_some() {
            return;
        }
        let n = self.y.len();
        let linha = self.tab.a[s - 2];
        let (mut num, mut den) = (0.0, 0.0);
        for i in 0..n {
            // y6 = y + h·Σ a₆ⱼ kⱼ (y é o início do passo; ainda não foi atualizado).
            let y6 = self.y[i]
                + hs * linha
                    .iter()
                    .enumerate()
                    .map(|(j, a)| a * self.k[j][i])
                    .sum::<f64>();
            num += (self.k[s - 1][i] - self.k[s - 2][i]).powi(2);
            den += (self.y_new[i] - y6).powi(2);
        }
        if den > 0.0 && hs.abs() * (num / den).sqrt() > 3.25 {
            self.nao_rigidos = 0;
            self.rigidos += 1;
            if self.rigidos == 15 {
                self.rigidez = Some(self.t + hs);
            }
        } else {
            self.nao_rigidos += 1;
            if self.nao_rigidos == 6 {
                self.rigidos = 0;
            }
        }
    }

    /// Norma RMS de eᵢ/(atolᵢ + rtol·max(|yᵢ|, |ŷᵢ|)).
    fn error_norm(&self) -> f64 {
        let n = self.y.len();
        let s: f64 = (0..n)
            .map(|i| {
                let sc = self.atol[i] + self.rtol * self.y[i].abs().max(self.y_new[i].abs());
                if sc == 0.0 {
                    if self.err[i] == 0.0 { 0.0 } else { f64::INFINITY }
                } else {
                    (self.err[i] / sc).powi(2)
                }
            })
            .sum();
        (s / n as f64).sqrt()
    }
}

impl Stepper for RkStepper {
    fn stiffness_detected(&self) -> Option<f64> {
        self.rigidez
    }

    fn t(&self) -> f64 {
        self.t
    }

    fn y(&self) -> &[f64] {
        &self.y
    }

    fn step(&mut self, f: &mut Rhs<'_>, stats: &mut OdeStats) -> Outcome {
        let s = self.tab.c.len();
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
            stats.fun_evals += step(
                self.tab,
                f,
                self.t,
                &self.y,
                hs,
                &mut self.k,
                &mut self.y_new,
                &mut self.err,
            );
            let finite = self
                .y_new
                .iter()
                .chain(&self.k[s - 1])
                .chain(&self.err)
                .all(|v| v.is_finite());
            if !finite {
                stats.rejected += 1;
                self.nonfinite_last = true;
                self.rejected_last = true;
                self.h = h * 0.25;
                continue;
            }
            self.nonfinite_last = false;
            let en = self.error_norm();
            if en > 1.0 {
                stats.rejected += 1;
                self.rejected_last = true;
                self.h = h * (0.9 * en.powf(-self.expo)).clamp(0.2, 1.0);
                continue;
            }
            stats.steps += 1;
            self.testa_rigidez(hs);
            let coeffs = dense_coeffs(self.tab, &self.y, &self.y_new, &self.k, hs);
            let t_old = self.t;
            self.t = if last { self.t_bound } else { self.t + hs };
            self.y.copy_from_slice(&self.y_new);
            let (first, rest) = self.k.split_at_mut(1);
            first[0].copy_from_slice(&rest[s - 2]);
            // Controle PI.
            let fac = if en == 0.0 {
                10.0
            } else {
                (0.9 * en.powf(-self.expo) * self.facold.powf(self.tab.beta)).clamp(0.2, 10.0)
            };
            self.facold = en.max(1e-4);
            self.h = h * if self.rejected_last { fac.min(1.0) } else { fac };
            self.rejected_last = false;
            return Outcome::Accepted(Interp::Rk {
                t: t_old,
                h: hs,
                coeffs,
            });
        }
    }
}
