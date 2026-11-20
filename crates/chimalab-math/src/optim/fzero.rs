//! Zero de função escalar: Brent–Dekker (Brent, 1973, cap. 4) e ITP (Oliveira & Takahashi,
//! ACM TOMS 47(1), 2020), com busca do intervalo a partir de um chute.

use super::{ExitFlag, OptimError, OptimResult, Options, Procedure, Tracker};

/// Método do zero escalar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ZeroMethod {
    /// Brent–Dekker: bissecção, secante e interpolação quadrática inversa.
    #[default]
    Brent,
    /// ITP: nunca usa mais iterações que a bissecção mais uma, e converge superlinearmente
    /// para raízes simples. Tolerância absoluta: termina com intervalo de largura ≤ 2ε,
    /// ε = max(tol_x/2, ε_máq·max(|a|, |b|)).
    Itp,
}

/// Ponto de partida de [`fzero`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    /// Intervalo [a, b] com f(a) e f(b) de sinais opostos (ou um deles zero).
    Interval(f64, f64),
    /// Chute: o intervalo é buscado em passos geométricos dos dois lados de x₀.
    Guess(f64),
}

/// Zero de `f` (raiz de uma função escalar contínua com mudança de sinal).
///
/// Padrões: `tol_x` = 0 (precisão total: o intervalo final tem largura ~4·ε_máq·|x|),
/// `max_iter` = 1000, `max_fun_evals` = 2000.
///
/// - Se `f` muda de sinal num polo ou num salto, o intervalo converge para ele e o resultado
///   sai com [`ExitFlag::SignChangeNotRoot`] (|f| final ≥ max(|f(a)|, |f(b)|) iniciais).
/// - Na busca a partir de um chute, um lado onde `f` é NaN ou ±∞ é abandonado e a busca segue
///   pelo outro.
/// - Um valor NaN dentro do intervalo termina com [`ExitFlag::NonFinite`].
///
/// Erros: `intervaloInvalido` (extremo não finito, a > b), `pontoInicialInvalido` (chute
/// não finito ou f(x₀) não finito), `semMudancaDeSinal`, `toleranciaInvalida`.
pub fn fzero(
    mut f: impl FnMut(f64) -> f64,
    start: Start,
    opts: &mut Options<'_>,
) -> Result<OptimResult<f64>, OptimError> {
    let tol_x = opts.tol_x.unwrap_or(0.0);
    let method = opts.zero_method;
    let mut t = Tracker::new(opts, 1000, 2000)?;
    let mut eval = |t: &mut Tracker, x: f64| {
        t.fun_evals += 1;
        f(x)
    };
    let (a, b, fa, fb) = match start {
        Start::Interval(a, b) => {
            if !(a.is_finite() && b.is_finite()) {
                return Err(OptimError::InvalidInterval("extremos devem ser finitos"));
            }
            if a > b {
                return Err(OptimError::InvalidInterval("a > b"));
            }
            let fa = eval(&mut t, a);
            let fb = if b == a { fa } else { eval(&mut t, b) };
            (a, b, fa, fb)
        }
        Start::Guess(x0) => {
            if !x0.is_finite() {
                return Err(OptimError::InvalidStart("chute não finito"));
            }
            let f0 = eval(&mut t, x0);
            if !f0.is_finite() {
                return Err(OptimError::InvalidStart("f(x0) não é finito"));
            }
            match search(&mut eval, &mut t, x0, f0)? {
                Found::Bracket(a, b, fa, fb) => (a, b, fa, fb),
                Found::Stop(x, fx, flag) => return Ok(t.finish(x, fx, flag, None)),
            }
        }
    };
    // Raiz exata num extremo.
    for (x, fx) in [(a, fa), (b, fb)] {
        if fx == 0.0 {
            t.report(&[x], fx, Procedure::Initial, Some((a, b)));
            return Ok(t.finish(x, fx, ExitFlag::Converged, Some((x, x))));
        }
    }
    if fa.is_nan() || fb.is_nan() || (fa > 0.0) == (fb > 0.0) {
        return Err(OptimError::NoSignChange {
            interval: (a, b),
            values: (fa, fb),
        });
    }
    let (x0, f0) = if fa.abs() < fb.abs() { (a, fa) } else { (b, fb) };
    if let Some(flag) = t.report(&[x0], f0, Procedure::Initial, Some((a, b))) {
        return Ok(t.finish(x0, f0, flag, Some((a, b))));
    }
    let scale = fa.abs().max(fb.abs());
    let (x, fx, flag, iv) = match method {
        ZeroMethod::Brent => brent(&mut eval, &mut t, a, b, fa, fb, tol_x),
        ZeroMethod::Itp => itp(&mut eval, &mut t, a, b, fa, fb, tol_x),
    };
    // Mudança de sinal que não é raiz: ao convergir, |f| não diminuiu em relação aos extremos
    // iniciais (polo: cresce; salto: fica igual). Perto de uma raiz de função contínua,
    // |f| ≤ L·(largura do intervalo) e fica muito abaixo dos valores nos extremos.
    let flag = if flag == ExitFlag::Converged && fx.abs() >= scale {
        ExitFlag::SignChangeNotRoot
    } else {
        flag
    };
    Ok(t.finish(x, fx, flag, Some(iv)))
}

enum Found {
    Bracket(f64, f64, f64, f64),
    Stop(f64, f64, ExitFlag),
}

/// Busca do intervalo: passos h·φᵏ (φ = 1,6) para os dois lados, com h = |x₀|/50 (ou 1/50).
/// O intervalo devolvido é o par de pontos consecutivos de um lado com sinais opostos, o mais
/// apertado possível.
fn search(
    eval: &mut impl FnMut(&mut Tracker, f64) -> f64,
    t: &mut Tracker,
    x0: f64,
    f0: f64,
) -> Result<Found, OptimError> {
    if f0 == 0.0 {
        t.report(&[x0], f0, Procedure::Initial, None);
        return Ok(Found::Stop(x0, f0, ExitFlag::Converged));
    }
    let mut h = if x0 == 0.0 { 1.0 / 50.0 } else { x0.abs() / 50.0 };
    // Último ponto de cada lado (x, f) e se o lado ainda está ativo.
    let mut left = (x0, f0, true);
    let mut right = (x0, f0, true);
    loop {
        for (side, dir) in [(&mut left, -1.0), (&mut right, 1.0)] {
            if !side.2 {
                continue;
            }
            let x = x0 + dir * h;
            if !x.is_finite() || x == side.0 {
                side.2 = false;
                continue;
            }
            let fx = eval(t, x);
            if !fx.is_finite() {
                // Fora do domínio (ou polo): abandona o lado.
                side.2 = false;
                continue;
            }
            let prev = (side.0, side.1);
            *side = (x, fx, true);
            if fx == 0.0 || (fx > 0.0) != (prev.1 > 0.0) {
                let (a, b, fa, fb) = if dir < 0.0 {
                    (x, prev.0, fx, prev.1)
                } else {
                    (prev.0, x, prev.1, fx)
                };
                t.iterations += 1;
                t.report(&[x], fx, Procedure::Search, Some((a, b)));
                return Ok(Found::Bracket(a, b, fa, fb));
            }
        }
        t.iterations += 1;
        let (bx, bf) = if left.1.abs() < right.1.abs() {
            (left.0, left.1)
        } else {
            (right.0, right.1)
        };
        if let Some(flag) = t.report(&[bx], bf, Procedure::Search, Some((left.0, right.0))) {
            return Ok(Found::Stop(bx, bf, flag));
        }
        if !left.2 && !right.2 {
            return Err(OptimError::NoSignChange {
                interval: (left.0, right.0),
                values: (left.1, right.1),
            });
        }
        h *= 1.6;
    }
}

type ZeroOut = (f64, f64, ExitFlag, (f64, f64));

fn brent(
    eval: &mut impl FnMut(&mut Tracker, f64) -> f64,
    t: &mut Tracker,
    mut a: f64,
    mut b: f64,
    mut fa: f64,
    mut fb: f64,
    tol_x: f64,
) -> ZeroOut {
    // b: melhor aproximação; c: extremo oposto (f(b)·f(c) < 0); a: aproximação anterior.
    let mut c = a;
    let mut fc = fa;
    let mut d = b - a;
    let mut e = d;
    loop {
        if (fb > 0.0) == (fc > 0.0) {
            c = a;
            fc = fa;
            d = b - a;
            e = d;
        }
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        let tol = 2.0 * f64::EPSILON * b.abs() + 0.5 * tol_x;
        let m = 0.5 * (c - b);
        let iv = (b.min(c), b.max(c));
        if m.abs() <= tol || fb == 0.0 {
            return (b, fb, ExitFlag::Converged, iv);
        }
        let mut procedure = Procedure::Bisection;
        // Ponto da bissecção na ordem dos floats, atribuído diretamente: b + (alvo − b) pode
        // arredondar e perder o alvo inteiro quando |alvo| ≪ |b|.
        let mut target = None;
        if e.abs() < tol || fa.abs() <= fb.abs() {
            (d, target) = bisect_step(b, c, m);
            e = d;
        } else {
            let s = fb / fa;
            let (mut p, mut q);
            if a == c {
                // Secante.
                p = 2.0 * m * s;
                q = 1.0 - s;
            } else {
                // Interpolação quadrática inversa.
                let qq = fa / fc;
                let r = fb / fc;
                p = s * (2.0 * m * qq * (qq - r) - (b - a) * (r - 1.0));
                q = (qq - 1.0) * (r - 1.0) * (s - 1.0);
            }
            if p > 0.0 {
                q = -q;
            } else {
                p = -p;
            }
            if 2.0 * p < 3.0 * m * q - (tol * q).abs() && p < (0.5 * e * q).abs() {
                e = d;
                d = p / q;
                procedure = Procedure::Interpolation;
            } else {
                (d, target) = bisect_step(b, c, m);
                e = d;
            }
        }
        a = b;
        fa = fb;
        b = match target {
            Some(t) if d.abs() > tol => t,
            _ => b + if d.abs() > tol { d } else { tol.copysign(m) },
        };
        fb = eval(t, b);
        t.iterations += 1;
        if fb.is_nan() {
            return (b, fb, ExitFlag::NonFinite, iv);
        }
        let iv = if (fb > 0.0) == (fc > 0.0) {
            (a.min(b), a.max(b))
        } else {
            (b.min(c), b.max(c))
        };
        if let Some(flag) = t.report(&[b], fb, procedure, Some(iv)) {
            return (b, fb, flag, iv);
        }
    }
}

/// Passo de bissecção de b em direção a c. Se o intervalo cruza o zero ou abrange mais de 10
/// ordens binárias, usa o ponto médio na ordem dos números de ponto flutuante (média das
/// representações inteiras): com tolerância relativa, um polo ou salto em 0 exigiria ~1075
/// bissecções aritméticas até os subnormais, e assim são no máximo ~64.
fn bisect_step(b: f64, c: f64, m: f64) -> (f64, Option<f64>) {
    let (lo, hi) = (b.abs().min(c.abs()), b.abs().max(c.abs()));
    if (b > 0.0) == (c > 0.0) && b != 0.0 && c != 0.0 && hi <= 1024.0 * lo {
        return (m, None);
    }
    let key = |x: f64| -> i64 {
        let i = x.to_bits() as i64;
        if i < 0 { i64::MIN - i } else { i }
    };
    let (kb, kc) = (key(b), key(c));
    let km = kb / 2 + kc / 2 + (kb % 2 + kc % 2) / 2;
    let mid = if km < 0 {
        f64::from_bits((i64::MIN - km) as u64)
    } else {
        f64::from_bits(km as u64)
    };
    if mid == b || mid == c {
        (m, None)
    } else {
        (mid - b, Some(mid))
    }
}

fn itp(
    eval: &mut impl FnMut(&mut Tracker, f64) -> f64,
    t: &mut Tracker,
    a0: f64,
    b0: f64,
    fa0: f64,
    fb0: f64,
    tol_x: f64,
) -> ZeroOut {
    // Normaliza para f(a) < 0 < f(b).
    let sg = if fa0 < 0.0 { 1.0 } else { -1.0 };
    let (mut a, mut b, mut ya, mut yb) = (a0, b0, sg * fa0, sg * fb0);
    let eps = (0.5 * tol_x).max(f64::EPSILON * a0.abs().max(b0.abs()));
    let eps = eps.max(f64::MIN_POSITIVE);
    // Parâmetros recomendados no artigo: κ₂ = 2, n₀ = 1; κ₁ = 0,2/(b₀ − a₀) para invariância de escala.
    let k1 = 0.2 / (b0 - a0);
    let n_half = ((b0 - a0) / (2.0 * eps)).log2().ceil().max(0.0);
    let n_max = n_half + 1.0;
    let best = |a: f64, b: f64, ya: f64, yb: f64| {
        if ya.abs() <= yb.abs() {
            (a, sg * ya)
        } else {
            (b, sg * yb)
        }
    };
    let mut j = 0.0;
    while b - a > 2.0 * eps {
        let half = 0.5 * (a + b);
        if half <= a || half >= b {
            break; // a e b vizinhos em ponto flutuante
        }
        let r = eps * (n_max - j).exp2() - 0.5 * (b - a);
        let delta = k1 * (b - a) * (b - a);
        let xf = (yb * a - ya * b) / (yb - ya);
        let sigma = (half - xf).signum();
        let xt = if delta <= (half - xf).abs() {
            xf + sigma * delta
        } else {
            half
        };
        let mut x = if (xt - half).abs() <= r {
            xt
        } else {
            half - sigma * r
        };
        if !(x > a && x < b) {
            x = half;
        }
        let y = sg * eval(t, x);
        t.iterations += 1;
        j += 1.0;
        if y.is_nan() {
            return (x, y, ExitFlag::NonFinite, (a, b));
        }
        if y > 0.0 {
            b = x;
            yb = y;
        } else if y < 0.0 {
            a = x;
            ya = y;
        } else {
            t.report(&[x], 0.0, Procedure::Itp, Some((x, x)));
            return (x, 0.0, ExitFlag::Converged, (x, x));
        }
        let (bx, bf) = best(a, b, ya, yb);
        if let Some(flag) = t.report(&[bx], bf, Procedure::Itp, Some((a, b))) {
            return (bx, bf, flag, (a, b));
        }
    }
    let (bx, bf) = best(a, b, ya, yb);
    (bx, bf, ExitFlag::Converged, (a, b))
}
