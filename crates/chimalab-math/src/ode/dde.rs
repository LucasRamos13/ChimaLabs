//! Equações diferenciais com retardos constantes: y'(t) = f(t, y(t), y(t − τ₁), …, y(t − τₖ)),
//! com y(t) = histórico(t) para t ≤ t₀ (funcionalidade F050).
//!
//! Método no estilo de L. F. Shampine & S. Thompson, Appl. Numer. Math. 37 (2001) 441–458:
//! - par de Bogacki–Shampine 3(2), com a Hermite cúbica de cada passo guardada como histórico
//!   para os valores atrasados;
//! - as descontinuidades da derivada propagadas a partir de t₀ (t₀ + combinações dos retardos com
//!   até 3 termos, a ordem do método) entram como pontos de parada, em que o passo cai exatamente
//!   e f é reavaliado (sem reaproveitar o último estágio);
//! - o passo é limitado ao menor retardo, de modo que os valores atrasados de um passo sempre vêm
//!   de passos já aceitos (sem iteração implícita). Retardos muito pequenos custam mais passos.

use super::rk::{self, BS23};
use super::{AbsTol, Dense, Interp, OdeError, OdeOptions, OdeSolution, OdeStats, OdeStatus, Rhs};

/// Integra o sistema com retardos `lags` (todos > 0) em [tspan[0], tspan[1]] (para a frente).
///
/// `f(t, y, z, dy)`: `z[j]` é y(t − lags[j]). `history(t, out)` preenche y(t) para t ≤ t₀.
/// Usa `rel_tol`, `abs_tol`, `max_step`, `initial_step` e `dense` de [`OdeOptions`]; o método é
/// sempre o BS23, e eventos não são aceitos neste incremento.
///
/// Erros: `intervaloInvalido` (dois instantes crescentes e finitos), `condicaoInicialInvalida`,
/// `toleranciaInvalida`, `opcaoInvalida` (retardo ≤ 0 ou não finito, eventos).
pub fn dde(
    mut f: impl FnMut(f64, &[f64], &[Vec<f64>], &mut [f64]),
    lags: &[f64],
    history: impl Fn(f64, &mut [f64]),
    tspan: [f64; 2],
    n: usize,
    opts: &mut OdeOptions<'_>,
) -> Result<OdeSolution, OdeError> {
    let [t0, tf] = tspan;
    if !(t0.is_finite() && tf.is_finite() && tf > t0) {
        return Err(OdeError::InvalidTspan("dde: t₀ < t_f, finitos"));
    }
    if lags.is_empty() || lags.iter().any(|l| !(l.is_finite() && *l > 0.0)) {
        return Err(OdeError::InvalidOption("dde: retardos finitos e > 0"));
    }
    if !opts.events.is_empty() {
        return Err(OdeError::InvalidOption("dde: eventos ainda não suportados"));
    }
    if n == 0 {
        return Err(OdeError::InvalidStart("estado vazio"));
    }
    let rtol_req = opts.rel_tol.unwrap_or(1e-6);
    // Validação explícita: NaN.max(…) devolveria o outro operando e esconderia o erro.
    if !(rtol_req.is_finite() && rtol_req >= 0.0) {
        return Err(OdeError::InvalidTolerance("relativa deve ser finita e ≥ 0"));
    }
    let rtol = rtol_req.max(100.0 * f64::EPSILON);
    let atol: Vec<f64> = match &opts.abs_tol {
        None => vec![1e-9; n],
        Some(AbsTol::Scalar(a)) => vec![*a; n],
        Some(AbsTol::Vector(v)) if v.len() == n => v.clone(),
        Some(AbsTol::Vector(v)) => {
            return Err(OdeError::ShapeMismatch {
                expected: n,
                actual: v.len(),
            });
        }
    };
    if atol.iter().any(|a| !(a.is_finite() && *a >= 0.0)) {
        return Err(OdeError::InvalidTolerance("absoluta"));
    }
    let mut y = vec![0.0; n];
    history(t0, &mut y);
    if y.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("histórico não finito em t₀"));
    }
    let min_lag = lags.iter().copied().fold(f64::INFINITY, f64::min);
    let hmax = opts.max_step.unwrap_or(tf - t0).min(min_lag).min(tf - t0);
    // Pontos de descontinuidade: t₀ + Σ mⱼτⱼ com Σ mⱼ ≤ 3.
    let mut quebras: Vec<f64> = Vec::new();
    let mut nivel = vec![0.0];
    for _ in 0..3 {
        let mut prox = Vec::new();
        for &b in &nivel {
            for &l in lags {
                prox.push(b + l);
            }
        }
        quebras.extend(prox.iter().copied());
        nivel = prox;
    }
    quebras.retain(|&d| t0 + d < tf);
    quebras.sort_by(f64::total_cmp);
    quebras.dedup_by(|a, b| (*a - *b).abs() <= 1e-12 * a.abs().max(1.0));
    let quebras: Vec<f64> = quebras.into_iter().map(|d| t0 + d).collect();
    let mut prox_quebra = 0usize;

    let mut segs: Vec<(f64, f64, Vec<f64>)> = Vec::new(); // (início, h, coeficientes)
    let atrasado = |segs: &[(f64, f64, Vec<f64>)], s: f64, out: &mut [f64]| {
        if s <= t0 {
            history(s, out);
            return;
        }
        let i = segs.partition_point(|seg| seg.0 <= s).saturating_sub(1);
        let (a, h, c) = &segs[i];
        rk::dense_eval(c, ((s - a) / h).clamp(0.0, 1.0), out);
    };
    let mut stats = OdeStats::default();
    let st = BS23.c.len();
    let mut k: Vec<Vec<f64>> = vec![vec![0.0; n]; st];
    let mut z: Vec<Vec<f64>> = vec![vec![0.0; n]; lags.len()];
    for (zj, l) in z.iter_mut().zip(lags) {
        atrasado(&segs, t0 - l, zj);
    }
    f(t0, &y, &z, &mut k[0]);
    stats.fun_evals += 1;
    if k[0].iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("f(t₀) não finito"));
    }
    let mut t = t0;
    let mut h = opts.initial_step.unwrap_or(0.01 * hmax).min(hmax);
    let mut out_t = vec![t0];
    let mut out_y = vec![y.clone()];
    let mut y_new = vec![0.0; n];
    let mut err = vec![0.0; n];
    let mut dense = Dense {
        n,
        starts: Vec::new(),
        interps: Vec::new(),
    };
    let status;
    loop {
        if opts.max_steps.is_some_and(|m| stats.steps >= m) {
            status = OdeStatus::MaxSteps;
            break;
        }
        let hmin = 16.0 * f64::EPSILON * t.abs().max(f64::MIN_POSITIVE);
        h = h.min(hmax);
        // Para exatamente na próxima descontinuidade ou no fim.
        let alvo = quebras.get(prox_quebra).copied().unwrap_or(tf).min(tf);
        let mut cai = false;
        if t + h * 1.01 >= alvo {
            h = alvo - t;
            cai = true;
        }
        if h < hmin {
            status = OdeStatus::StepTooSmall;
            break;
        }
        {
            let segs_ref = &segs;
            let mut rhs = |tt: f64, yy: &[f64], dy: &mut [f64]| {
                let mut zz: Vec<Vec<f64>> = vec![vec![0.0; n]; lags.len()];
                for (zj, l) in zz.iter_mut().zip(lags) {
                    atrasado(segs_ref, tt - l, zj);
                }
                f(tt, yy, &zz, dy);
            };
            let r: &mut Rhs<'_> = &mut rhs;
            stats.fun_evals += rk::step(&BS23, r, t, &y, h, &mut k, &mut y_new, &mut err);
        }
        let finite = y_new.iter().chain(&err).all(|v| v.is_finite());
        let en = if finite {
            let s: f64 = (0..n)
                .map(|i| {
                    let sc = atol[i] + rtol * y[i].abs().max(y_new[i].abs());
                    if sc == 0.0 {
                        if err[i] == 0.0 { 0.0 } else { f64::INFINITY }
                    } else {
                        (err[i] / sc).powi(2)
                    }
                })
                .sum();
            (s / n as f64).sqrt()
        } else {
            f64::INFINITY
        };
        if en > 1.0 {
            stats.rejected += 1;
            h *= if finite {
                (0.9 * en.powf(-1.0 / 3.0)).clamp(0.2, 1.0)
            } else {
                0.25
            };
            continue;
        }
        stats.steps += 1;
        let coeffs = rk::dense_coeffs(&BS23, &y, &y_new, &k, h);
        let t_new = if cai { alvo } else { t + h };
        segs.push((t, t_new - t, coeffs.clone()));
        dense.starts.push(t);
        dense.interps.push(Interp::Rk {
            t,
            h: t_new - t,
            coeffs,
        });
        t = t_new;
        y.copy_from_slice(&y_new);
        out_t.push(t);
        out_y.push(y.clone());
        if cai && prox_quebra < quebras.len() && t == quebras[prox_quebra] {
            prox_quebra += 1;
            // Descontinuidade da derivada: f é reavaliado (sem FSAL).
            for (zj, l) in z.iter_mut().zip(lags) {
                atrasado(&segs, t - l, zj);
            }
            f(t, &y, &z, &mut k[0]);
            stats.fun_evals += 1;
        } else {
            let (first, rest) = k.split_at_mut(1);
            first[0].copy_from_slice(&rest[st - 2]);
        }
        if t >= tf {
            status = OdeStatus::Completed;
            break;
        }
        let fac = if en == 0.0 {
            5.0
        } else {
            (0.9 * en.powf(-1.0 / 3.0)).clamp(0.2, 5.0)
        };
        h *= fac;
    }
    Ok(OdeSolution {
        t: out_t,
        y: out_y,
        events: Vec::new(),
        status,
        stats,
        warnings: Vec::new(),
        dense: opts.dense.then_some(dense),
    })
}

/// Lado direito com valores atrasados: `f(t, y, z, dy)`.
type FnAtraso<'a> = dyn FnMut(f64, &[f64], &[Vec<f64>], &mut [f64]) + 'a;

/// Retardos variáveis: y'(t) = f(t, y(t), y(s₁), …, y(sₖ)), com os instantes atrasados
/// sⱼ = `delays(t, y)` ≤ t dependentes de t e do estado, inclusive retardos que se anulam
/// (sⱼ → t, como na equação pantográfica y'(t) = y(t/2) a partir de t₀ = 0).
///
/// Mesmo par BS23 de [`dde`], mas o passo não é limitado pelo retardo. Quando um sⱼ cai dentro
/// do passo em curso, o valor vem do interpolante desse mesmo passo, obtido por iteração de
/// ponto fixo (A. Bellen & M. Zennaro, *Numerical Methods for Delay Differential Equations*,
/// Oxford, 2003, §4.1): a primeira passada extrapola o último passo aceito, as seguintes usam o
/// interpolante provisório, até a mudança ficar abaixo de 1% da tolerância (no máximo 8
/// passadas; sem convergência, o passo é reduzido). Não há rastreamento de descontinuidades: o
/// controle de erro reduz o passo perto delas.
///
/// `delays(t, y, s)` preenche os `nd` instantes atrasados. Erros: os de [`dde`], e
/// `opcaoInvalida` se algum sⱼ > t ou não for finito durante a integração.
#[allow(clippy::too_many_arguments)]
pub fn dde_var(
    mut f: impl FnMut(f64, &[f64], &[Vec<f64>], &mut [f64]),
    delays: impl Fn(f64, &[f64], &mut [f64]),
    nd: usize,
    history: impl Fn(f64, &mut [f64]),
    tspan: [f64; 2],
    n: usize,
    opts: &mut OdeOptions<'_>,
) -> Result<OdeSolution, OdeError> {
    let [t0, tf] = tspan;
    if !(t0.is_finite() && tf.is_finite() && tf > t0) {
        return Err(OdeError::InvalidTspan("dde: t₀ < t_f, finitos"));
    }
    if nd == 0 {
        return Err(OdeError::InvalidOption("dde: pelo menos um retardo"));
    }
    if !opts.events.is_empty() {
        return Err(OdeError::InvalidOption("dde: eventos ainda não suportados"));
    }
    if n == 0 {
        return Err(OdeError::InvalidStart("estado vazio"));
    }
    let rtol_req = opts.rel_tol.unwrap_or(1e-6);
    if !(rtol_req.is_finite() && rtol_req >= 0.0) {
        return Err(OdeError::InvalidTolerance("relativa deve ser finita e ≥ 0"));
    }
    let rtol = rtol_req.max(100.0 * f64::EPSILON);
    let atol: Vec<f64> = match &opts.abs_tol {
        None => vec![1e-9; n],
        Some(AbsTol::Scalar(a)) => vec![*a; n],
        Some(AbsTol::Vector(v)) if v.len() == n => v.clone(),
        Some(AbsTol::Vector(v)) => {
            return Err(OdeError::ShapeMismatch {
                expected: n,
                actual: v.len(),
            });
        }
    };
    if atol.iter().any(|a| !(a.is_finite() && *a >= 0.0)) {
        return Err(OdeError::InvalidTolerance("absoluta"));
    }
    let mut y = vec![0.0; n];
    history(t0, &mut y);
    if y.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("histórico não finito em t₀"));
    }
    let hmax = opts.max_step.unwrap_or(tf - t0).min(tf - t0);
    // Passos aceitos (início, h, coeficientes) e o passo provisório em curso.
    let mut segs: Vec<(f64, f64, Vec<f64>)> = Vec::new();
    let st = BS23.c.len();
    let mut k: Vec<Vec<f64>> = vec![vec![0.0; n]; st];
    let mut stats = OdeStats::default();
    let mut atraso_invalido = false;
    // Valor em s: histórico, passos aceitos, passo provisório (ou extrapolação do último passo,
    // ou Euler a partir de (t, y, y') no primeiro passo).
    let valor = |segs: &[(f64, f64, Vec<f64>)],
                 prov: Option<&(f64, f64, Vec<f64>)>,
                 base: (f64, &[f64], &[f64]),
                 s: f64,
                 out: &mut [f64]| {
        if s <= t0 {
            history(s, out);
            return;
        }
        let (tb, yb, db) = base;
        if s > tb {
            if let Some((a, h, c)) = prov {
                rk::dense_eval(c, (s - a) / h, out);
            } else if let Some((a, h, c)) = segs.last() {
                rk::dense_eval(c, (s - a) / h, out);
            } else {
                for ((o, yi), di) in out.iter_mut().zip(yb).zip(db) {
                    *o = yi + (s - tb) * di;
                }
            }
            return;
        }
        let i = segs.partition_point(|seg| seg.0 <= s).saturating_sub(1);
        let (a, h, c) = &segs[i];
        rk::dense_eval(c, ((s - a) / h).clamp(0.0, 1.0), out);
    };
    let mut sj = vec![0.0; nd];
    let mut z: Vec<Vec<f64>> = vec![vec![0.0; n]; nd];
    // f em (t, y) com os atrasados até t (sem passo provisório).
    let mut avalia_base = |f: &mut FnAtraso<'_>,
                           segs: &[(f64, f64, Vec<f64>)],
                           t: f64,
                           y: &[f64],
                           out: &mut [f64],
                           invalido: &mut bool| {
        delays(t, y, &mut sj);
        let zero = vec![0.0; n];
        for (zj, &s) in z.iter_mut().zip(&sj) {
            if !(s.is_finite() && s <= t) {
                *invalido = true;
            }
            valor(segs, None, (t, y, &zero), s.min(t), zj);
        }
        f(t, y, &z, out);
    };
    let mut k0 = vec![0.0; n];
    avalia_base(&mut f, &segs, t0, &y, &mut k0, &mut atraso_invalido);
    stats.fun_evals += 1;
    if atraso_invalido {
        return Err(OdeError::InvalidOption(
            "dde: instante atrasado > t ou não finito",
        ));
    }
    if k0.iter().any(|v| !v.is_finite()) {
        return Err(OdeError::InvalidStart("f(t₀) não finito"));
    }
    let mut t = t0;
    let mut h = opts.initial_step.unwrap_or(0.01 * hmax).min(hmax);
    let mut out_t = vec![t0];
    let mut out_y = vec![y.clone()];
    let mut y_new = vec![0.0; n];
    let mut err = vec![0.0; n];
    let mut dense = Dense {
        n,
        starts: Vec::new(),
        interps: Vec::new(),
    };
    let status;
    loop {
        if opts.max_steps.is_some_and(|m| stats.steps >= m) {
            status = OdeStatus::MaxSteps;
            break;
        }
        let hmin = 16.0 * f64::EPSILON * t.abs().max(f64::MIN_POSITIVE);
        h = h.min(hmax);
        let mut ultimo = false;
        if t + h * 1.01 >= tf {
            h = tf - t;
            ultimo = true;
        }
        if h < hmin {
            status = OdeStatus::StepTooSmall;
            break;
        }
        // Passadas de ponto fixo sobre o interpolante do passo.
        let mut prov: Option<(f64, f64, Vec<f64>)> = None;
        let mut convergiu = false;
        let mut anterior: Option<Vec<f64>> = None;
        for _ in 0..8 {
            let mut sobrepoe = false;
            {
                let segs_ref = &segs;
                let prov_ref = prov.as_ref();
                let (y_ref, k0_ref) = (&y, &k0);
                let mut sl = vec![0.0; nd];
                let mut rhs = |tt: f64, yy: &[f64], dy: &mut [f64]| {
                    delays(tt, yy, &mut sl);
                    let mut zz: Vec<Vec<f64>> = vec![vec![0.0; n]; nd];
                    for (zj, &s) in zz.iter_mut().zip(&sl) {
                        if !(s.is_finite() && s <= tt) {
                            atraso_invalido = true;
                        }
                        let s = s.min(tt);
                        if s > t {
                            sobrepoe = true;
                        }
                        valor(segs_ref, prov_ref, (t, y_ref, k0_ref), s, zj);
                    }
                    f(tt, yy, &zz, dy);
                };
                k[0].copy_from_slice(&k0);
                let r: &mut Rhs<'_> = &mut rhs;
                stats.fun_evals += rk::step(&BS23, r, t, &y, h, &mut k, &mut y_new, &mut err);
            }
            if atraso_invalido {
                return Err(OdeError::InvalidOption(
                    "dde: instante atrasado > t ou não finito",
                ));
            }
            if !sobrepoe {
                convergiu = true;
                break;
            }
            let mudou = anterior.as_ref().map(|a: &Vec<f64>| {
                (0..n)
                    .map(|i| (y_new[i] - a[i]).abs() / (atol[i] + rtol * y_new[i].abs()))
                    .fold(0.0, f64::max)
            });
            prov = Some((t, h, rk::dense_coeffs(&BS23, &y, &y_new, &k, h)));
            if mudou.is_some_and(|m| m <= 0.01) {
                convergiu = true;
                break;
            }
            anterior = Some(y_new.clone());
        }
        let finite = y_new.iter().chain(&err).all(|v| v.is_finite());
        let en = if !convergiu || !finite {
            f64::INFINITY
        } else {
            let s: f64 = (0..n)
                .map(|i| {
                    let sc = atol[i] + rtol * y[i].abs().max(y_new[i].abs());
                    if sc == 0.0 {
                        if err[i] == 0.0 { 0.0 } else { f64::INFINITY }
                    } else {
                        (err[i] / sc).powi(2)
                    }
                })
                .sum();
            (s / n as f64).sqrt()
        };
        if en > 1.0 {
            stats.rejected += 1;
            h *= if en.is_finite() {
                (0.9 * en.powf(-1.0 / 3.0)).clamp(0.2, 1.0)
            } else {
                0.25
            };
            continue;
        }
        stats.steps += 1;
        let coeffs = rk::dense_coeffs(&BS23, &y, &y_new, &k, h);
        let t_new = if ultimo { tf } else { t + h };
        segs.push((t, t_new - t, coeffs.clone()));
        dense.starts.push(t);
        dense.interps.push(Interp::Rk {
            t,
            h: t_new - t,
            coeffs,
        });
        t = t_new;
        y.copy_from_slice(&y_new);
        out_t.push(t);
        out_y.push(y.clone());
        // f no novo ponto com os atrasados já aceitos (sem FSAL: o último estágio usou o
        // interpolante provisório).
        avalia_base(&mut f, &segs, t, &y, &mut k0, &mut atraso_invalido);
        stats.fun_evals += 1;
        if atraso_invalido {
            return Err(OdeError::InvalidOption(
                "dde: instante atrasado > t ou não finito",
            ));
        }
        if t >= tf {
            status = OdeStatus::Completed;
            break;
        }
        let fac = if en == 0.0 {
            5.0
        } else {
            (0.9 * en.powf(-1.0 / 3.0)).clamp(0.2, 5.0)
        };
        h *= fac;
    }
    Ok(OdeSolution {
        t: out_t,
        y: out_y,
        events: Vec::new(),
        status,
        stats,
        warnings: Vec::new(),
        dense: opts.dense.then_some(dense),
    })
}
