//! Mínimo de função escalar num intervalo: seção áurea com interpolação parabólica
//! (R. P. Brent, *Algorithms for Minimization without Derivatives*, 1973, cap. 5).

use super::{ExitFlag, OptimError, OptimResult, Options, Procedure, Tracker};

/// Mínimo de `f` em [a, b].
///
/// Converge para um mínimo local; para funções unimodais, para o mínimo global. A precisão em x
/// é √ε_máq·|x| + tol_x/3: perto de um mínimo suave, f varia com o quadrado do erro em x, e
/// mais precisão em x não é atingível em ponto flutuante.
///
/// Padrões: `tol_x` = 1e-10, `max_iter` = 500, `max_fun_evals` = 1000.
///
/// O método nunca avalia os extremos. Ao final, eles são avaliados (2 avaliações), e o resultado
/// é o extremo se f lá for menor ([`Procedure::Boundary`] no histórico): o mínimo de uma função
/// monótona no intervalo fica exato. NaN conta como +∞.
///
/// Erros: `intervaloInvalido` (extremo não finito ou a > b), `toleranciaInvalida`.
pub fn fminbnd(
    mut f: impl FnMut(f64) -> f64,
    a: f64,
    b: f64,
    opts: &mut Options<'_>,
) -> Result<OptimResult<f64>, OptimError> {
    if !(a.is_finite() && b.is_finite()) {
        return Err(OptimError::InvalidInterval("extremos devem ser finitos"));
    }
    if a > b {
        return Err(OptimError::InvalidInterval("a > b"));
    }
    let tol_x = opts.tol_x.unwrap_or(1e-10);
    let mut t = Tracker::new(opts, 500, 1000)?;
    let mut eval = |t: &mut Tracker, x: f64| {
        t.fun_evals += 1;
        let y = f(x);
        if y.is_nan() { f64::INFINITY } else { y }
    };
    let (a0, b0) = (a, b);
    let (mut a, mut b) = (a, b);
    let c = 0.5 * (3.0 - 5f64.sqrt());
    let sqeps = f64::EPSILON.sqrt();
    let mut x = a + c * (b - a);
    let (mut v, mut w) = (x, x);
    let mut d = 0.0f64;
    let mut e = 0.0f64;
    let mut fx = eval(&mut t, x);
    let (mut fv, mut fw) = (fx, fx);
    let mut exit = t.report(&[x], fx, Procedure::Initial, Some((a, b)));
    while exit.is_none() {
        let m = 0.5 * (a + b);
        let tol = sqeps * x.abs() + tol_x / 3.0;
        let t2 = 2.0 * tol;
        if (x - m).abs() <= t2 - 0.5 * (b - a) {
            exit = Some(ExitFlag::Converged);
            break;
        }
        let mut parabolic = false;
        // Com f infinito em x, v ou w, p e q dão ±∞ ou NaN e os testes abaixo falham: o passo
        // cai na seção áurea sem precisar de checagem explícita.
        if parabola_allowed(e, tol) {
            let mut r = (x - w) * (fx - fv);
            let mut q = (x - v) * (fx - fw);
            let mut p = (x - v) * q - (x - w) * r;
            q = 2.0 * (q - r);
            if q > 0.0 {
                p = -p;
            } else {
                q = -q;
            }
            r = e;
            e = d;
            if p.abs() < (0.5 * q * r).abs() && p > q * (a - x) && p < q * (b - x) {
                d = p / q;
                let u = x + d;
                if u - a < t2 || b - u < t2 {
                    d = tol.copysign(m - x);
                }
                parabolic = true;
            }
        }
        if !parabolic {
            e = if x < m { b - x } else { a - x };
            d = c * e;
        }
        let u = x + if d.abs() >= tol { d } else { tol.copysign(d) };
        let fu = eval(&mut t, u);
        t.iterations += 1;
        if fu <= fx {
            if u < x {
                b = x;
            } else {
                a = x;
            }
            (v, fv, w, fw, x, fx) = (w, fw, x, fx, u, fu);
        } else {
            if u < x {
                a = u;
            } else {
                b = u;
            }
            if fu <= fw || w == x {
                (v, fv, w, fw) = (w, fw, u, fu);
            } else if fu <= fv || v == x || v == w {
                (v, fv) = (u, fu);
            }
        }
        let proc = if parabolic {
            Procedure::Parabolic
        } else {
            Procedure::Golden
        };
        exit = t.report(&[x], fx, proc, Some((a, b)));
    }
    let exit = exit.unwrap_or(ExitFlag::Converged);
    if exit != ExitFlag::Stopped {
        // Extremos: o método não os avalia.
        for xe in [a0, b0] {
            let fe = eval(&mut t, xe);
            if fe < fx {
                x = xe;
                fx = fe;
                t.report(&[x], fx, Procedure::Boundary, Some((a, b)));
            }
        }
    }
    Ok(t.finish(x, fx, exit, Some((a, b))))
}

/// Brent: só tenta a parábola se o penúltimo passo foi maior que a tolerância; caso contrário
/// os passos poderiam encolher sem que o intervalo encolhesse.
fn parabola_allowed(e: f64, tol: f64) -> bool {
    e.abs() > tol
}
